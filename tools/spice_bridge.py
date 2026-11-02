import ctypes
import os
from pathlib import Path
import sys


def main():
    source, destination = sys.argv[1:3]
    directory = Path(sys.executable).parent
    library_path = directory / "ngspice.dll"
    if not library_path.is_file():
        raise RuntimeError("KiCad bundled ngspice.dll not found; configure ngspice executable")
    dll_directory = os.add_dll_directory(str(directory)) if os.name == "nt" else None
    library = ctypes.CDLL(str(library_path))
    messages = []
    output_type = ctypes.CFUNCTYPE(ctypes.c_int, ctypes.c_char_p, ctypes.c_int, ctypes.c_void_p)
    exit_type = ctypes.CFUNCTYPE(ctypes.c_int, ctypes.c_int, ctypes.c_bool, ctypes.c_bool, ctypes.c_int, ctypes.c_void_p)

    @output_type
    def output(message, identifier, userdata):
        text = message.decode("utf-8", errors="replace")
        messages.append(text.removeprefix("stdout ").removeprefix("stderr "))
        return 0

    @exit_type
    def controlled_exit(status, immediate, quitting, identifier, userdata):
        messages.append(f"ngspice exit status: {status}")
        return 0

    library.ngSpice_Init.argtypes = [output_type, ctypes.c_void_p, exit_type, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p]
    library.ngSpice_Init.restype = ctypes.c_int
    library.ngSpice_Command.argtypes = [ctypes.c_char_p]
    library.ngSpice_Command.restype = ctypes.c_int
    if library.ngSpice_Init(output, None, controlled_exit, None, None, None, None) != 0:
        raise RuntimeError("ngspice initialization failed")
    failed = False
    for command in ["version", f"source {source}", "run"]:
        failed |= library.ngSpice_Command(command.encode("utf-8")) != 0
    Path(destination).write_text("\n".join(messages) + "\n", encoding="utf-8")
    if dll_directory:
        dll_directory.close()
    if failed:
        raise RuntimeError("ngspice command failed; inspect simulation report")


if __name__ == "__main__":
    main()
