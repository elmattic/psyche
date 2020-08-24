import itertools
import lldb
import os
import re
import shlex
import struct
import subprocess
import sys
from cmd import Cmd
from collections import namedtuple, OrderedDict
try:
    # Python 2
    from future_builtins import filter
except ImportError:
    # Python 3
    pass

Breakpoint = namedtuple('Breakpoint', 'cond id')


class Disassembly(object):
    def __init__(self, frame):
        bytecode_var = frame.FindVariable("bytecode")
        data_ptr_var = bytecode_var.GetChildMemberWithName("data_ptr")
        length_var = bytecode_var.GetChildMemberWithName("length")
        bytecode = data_ptr_var.GetPointeeData(0, int(length_var.GetValue()))
        hexstr = "".join(map(lambda b: format(b, "02x"), bytecode.uint8))
        args = ["./target/debug/psyche", "disasm", hexstr]
        process = subprocess.Popen(args, stdout=subprocess.PIPE)
        (output, error) = process.communicate()
        lines = output.decode("utf-8").split("\n")
        self.instructions = OrderedDict()
        for line in lines:
            if line:
                tu = line.split(":")
                self.instructions[int(tu[0], 16)] = line


class EVMDCmd(Cmd):
    prompt = '(evmd) '

    def __init__(self, debugger):
        Cmd.__init__(self)
        self.debugger = debugger
        self.disasm = None
        self.code_ptr = None
        self.stack_ptr_var = None
        self.stack_ptr = None
        self.bp_single_step_id = None
        self.bp_stop_id = None
        self.breakpoints = []
        self.hex_stack_format = False
        self._save_lldb_state()

    def __del__(self):
        self._restore_lldb_state()

        target = self.debugger.GetSelectedTarget()
        for bp in self.breakpoints:
            target.BreakpointDelete(bp.id)
        target.BreakpointDelete(self.bp_single_step_id)
        target.BreakpointDelete(self.bp_stop_id)

    def _save_lldb_state(self):
        devnull = open(os.devnull, "w")
        self.debugger.SetOutputFileHandle(devnull, True)

    def _restore_lldb_state(self):
        self.debugger.SetOutputFileHandle(sys.stdout, False)

    def _create_breakpoint(self, addr, cond=None):
        target = self.debugger.GetSelectedTarget()
        breakpoint = target.BreakpointCreateByName("lldb_hook_single_step")
        c = ""
        if addr:
            c = "(pc == 0x{:x})".format(addr)
        if cond:
            c = "|| ({})".format(cond) if c else cond
        breakpoint.SetCondition(c)
        self.breakpoints.append(Breakpoint(c, breakpoint.GetID()))

    def _u64_array_to_int(self, value):
        result = int(value.GetChildAtIndex(0).GetValue())
        result += int(value.GetChildAtIndex(1).GetValue()) << 64
        result += int(value.GetChildAtIndex(2).GetValue()) << 128
        result += int(value.GetChildAtIndex(3).GetValue()) << 192
        return result

    def _print_state(self, frame):
        if self.disasm is None:
            self.disasm = Disassembly(frame)

        # print machine registers (pc, gas, stsize, stack)
        stack_start_var = frame.FindVariable("stack_start")
        stack_size = int(frame.FindVariable("ssize").GetValue())
        memory_size = int(frame.FindVariable("msize").GetValue())
        stackdata = stack_start_var.GetPointeeData(0, stack_size * 32)
        bytes_str = b"".join(map(lambda x: bytes(bytearray([x])), stackdata.uint8))
        stack = []
        for i in range(stack_size):
            value = 0
            for j in range(4):
                offset = i*32 + j*8
                temp = bytes_str[offset:(offset+8)]
                x = struct.unpack("<Q", temp)
                value = value + (x[0] << (j*64))
            stack.append(value)
        stack_str = ""
        slots = 16
        if self.hex_stack_format:
            formatter = lambda v: "0x{:064x}".format(v)
        else:
            formatter = lambda v: str(v)
        if stack_size > slots:
            stack_str += "[..., " + ", ".join(map(formatter, stack[-slots:])) + "]"
        else:
            stack_str += "[" + ", ".join(map(formatter, stack)) + "]"
        pc = int(frame.FindVariable("pc").GetValue())
        #arr = frame.FindVariable("gas").GetChildAtIndex(0)
        #gas = self._u64_array_to_int(arr)
        gas = int(frame.FindVariable("gas").GetValue())
        hud_str = "pc: {:04x}    gas: {:,}    ssize: {}    msize: {}\nstack: {}"
        print(hud_str.format(pc, gas, stack_size, memory_size, stack_str))

        # print instructions window
        index = list(self.disasm.instructions).index(pc)
        offset = max(0, index - 4)
        window = list(self.disasm.instructions)[offset:]
        def to_str(k):
            gutter = "-> " if k == pc else "   "
            s = gutter + self.disasm.instructions[k]
            return s
        wsize = 17
        print("\n".join(map(to_str, itertools.islice(window, wsize))))

    def _print_breakpoints(self, target):
        def to_str(bp):
            hits = target.FindBreakpointByID(bp.id).GetHitCount()
            fmt_str = "{}: condition = '{}', hit count = {}"
            return fmt_str.format(bp.id, bp.cond, hits)
        if self.breakpoints:
            print("Current breakpoints:")
            print("\n".join(map(to_str, self.breakpoints)))
        else:
            print("No breakpoints currently set.")

    def default(self, line):
        line2 = re.sub(r"#.*", "", line)
        if line2 == "":
            return
        cmd, arg, line = self.parseline(line2)
        func = [getattr(self, n) for n in self.get_names() if n.startswith('do_' + cmd)]
        if func:
            func[0](arg)
        else:
            print("error: '{}' is not a valid command.".format(line2))

    def do_run(self, arg):
        'Launch the executable in LLDB.'

        args = arg.split(" ")
        target = self.debugger.GetSelectedTarget()
        process = target.LaunchSimple(args, None, os.getcwd())
        assert process.state == lldb.eStateStopped

        frame = None
        for thread in process:
            ID = thread.GetThreadID()
            if thread.GetStopReason() == lldb.eStopReasonBreakpoint:
                for f in thread:
                    assert f.GetThread().GetThreadID() == ID
                    frame = f.get_parent_frame()
                    break
        assert frame
        self._print_state(frame)

    def do_next(self, arg):
        'EVM level single step, stepping over calls.'

        target = self.debugger.GetSelectedTarget()
        bp_single_step = target.FindBreakpointByID(self.bp_single_step_id)
        bp_single_step.SetEnabled(True)

        process = target.GetProcess()
        err = process.Continue()
        assert err.Success()

        frame = None
        for thread in process:
            ID = thread.GetThreadID()
            if thread.GetStopReason() == lldb.eStopReasonBreakpoint:
                for f in thread:
                    assert f.GetThread().GetThreadID() == ID
                    frame = f.get_parent_frame()
                    break
        assert frame
        self._print_state(frame)

    def do_breakpoint(self, arg):
        'Commands for operating on breakpoints.'

        target = self.debugger.GetSelectedTarget()
        args = shlex.split(arg)
        if len(args) == 0:
            self._print_breakpoints(target)
        elif len(args) == 1:
            if args[0] == 'list':
                self._print_breakpoints(target)
                return
            try:
                addr = int(args[0], 16)
                self._create_breakpoint(addr)
            except ValueError:
                print("error: Argument must be an address.")
        elif len(args) >= 2:
            if args[0] == 'delete':
                id = None
                try:
                    id = int(args[1], 10)
                except ValueError:
                    print("error: Argument must be a breakpoint id.")
                    return
                bp = next(filter(lambda bp: bp.id == id, self.breakpoints), None)
                if bp:
                    target.BreakpointDelete(id)
                    self.breakpoints.remove(bp)
                else:
                    print("error: No breakpoints exist to be deleted.")
            elif args[0] == 'set':
                opt = "-c"
                index = args.index(opt) if opt in args else None
                if index:
                    try:
                        arg = args[index + 1]
                        self._create_breakpoint(None, arg)
                    except IndexError:
                        print("error: Missing option argument")

    def do_continue(self, arg):
        'Continue execution of all threads in the current process.'

        target = self.debugger.GetSelectedTarget()
        bp_single_step = target.FindBreakpointByID(self.bp_single_step_id)
        bp_single_step.SetEnabled(False)

        process = target.GetProcess()
        err = process.Continue()
        assert err.Success()

        frame = None
        for thread in process:
            ID = thread.GetThreadID()
            if thread.GetStopReason() == lldb.eStopReasonBreakpoint:
                for f in thread:
                    assert f.GetThread().GetThreadID() == ID
                    frame = f.get_parent_frame()
                    break
        assert frame
        self._print_state(frame)

    def do_hex(self, arg):
        'Toggle stack formatting between decimal and hexadecimal.'
        self.hex_stack_format = not self.hex_stack_format

        target = self.debugger.GetSelectedTarget()
        process = target.GetProcess()

        frame = None
        for thread in process:
            ID = thread.GetThreadID()
            if thread.GetStopReason() == lldb.eStopReasonBreakpoint:
                for f in thread:
                    assert f.GetThread().GetThreadID() == ID
                    frame = f.get_parent_frame()
                    break
        assert frame
        self._print_state(frame)

    def do_quit(self, arg):
        'Quit the EVMD prompt and return to the LLDB prompt.'

        quit()


def run_evmd(debugger, command, result, internal_dict):
    evmd = EVMDCmd(debugger)

    target = debugger.GetSelectedTarget()
    bp_single_step = target.BreakpointCreateByName("lldb_hook_single_step")
    bp_stop = target.BreakpointCreateByName("lldb_hook_stop")

    evmd.bp_single_step_id = bp_single_step.GetID()
    evmd.bp_stop_id = bp_stop.GetID()

    try:
        evmd.cmdloop()
    except Exception as e:
        print(e)
        pass


def __lldb_init_module(debugger, internal_dict):
    debugger.HandleCommand("command script add -f evmd.run_evmd evmd")
