"""KiCad pcbnew bridge. Invoked only with compiler-generated project files."""
import json
import sys
import pcbnew


def main():
    mode, board_path, exchange_path = sys.argv[1:4]
    board = pcbnew.LoadBoard(board_path)
    if not board:
        raise RuntimeError("Cannot load PCB")
    if mode == "export":
        if not pcbnew.ExportSpecctraDSN(board, exchange_path):
            raise RuntimeError("Specctra export failed")
    elif mode == "import":
        if not pcbnew.ImportSpecctraSES(board, exchange_path):
            raise RuntimeError("Specctra import failed")
        pcbnew.SaveBoard(board_path, board)
    elif mode == "inspect":
        pads = sorted((fp.GetReference(), p.GetNumber(), p.GetNetname())
                      for fp in board.GetFootprints() for p in fp.Pads())
        expected = sorted([
            ("J1", "1", "IN"), ("J1", "2", "GND"),
            ("R1", "1", "IN"), ("R1", "2", "OUT"),
            ("C1", "1", "OUT"), ("C1", "2", "GND"),
            ("J2", "1", "OUT"), ("J2", "2", "GND"),
        ])
        if pads != expected:
            raise RuntimeError("PCB pin-level netlist mismatch")
        pad_geometry = []
        for footprint in board.GetFootprints():
            for pad in footprint.Pads():
                position = pad.GetPosition()
                size = pad.GetSize()
                pad_geometry.append({
                    "reference": footprint.GetReference(), "number": pad.GetNumber(),
                    "net": pad.GetNetname(), "x": pcbnew.ToMM(position.x), "y": pcbnew.ToMM(position.y),
                    "width": pcbnew.ToMM(size.x), "height": pcbnew.ToMM(size.y),
                    "drill": pcbnew.ToMM(pad.GetDrillSize().x), "rect": pad.GetShape() == pcbnew.PAD_SHAPE_RECT,
                })
        routes = []
        for track in board.GetTracks():
            start, end = track.GetStart(), track.GetEnd()
            routes.append({
                "net": track.GetNetname(), "layer": "back" if track.GetLayer() == pcbnew.B_Cu else "front",
                "x1": pcbnew.ToMM(start.x), "y1": pcbnew.ToMM(start.y),
                "x2": pcbnew.ToMM(end.x), "y2": pcbnew.ToMM(end.y),
                "width": pcbnew.ToMM(track.GetWidth()), "via": isinstance(track, pcbnew.PCB_VIA),
            })
        with open(exchange_path, "w", encoding="utf-8") as file:
            json.dump({"pads": pads, "tracks": len(routes), "pad_geometry": pad_geometry, "routes": routes}, file)
    elif mode == "netlist":
        import xml.etree.ElementTree as ET
        root = ET.parse(exchange_path).getroot()
        actual = sorted((n.attrib["ref"], n.attrib["pin"], net.attrib["name"].split("/")[-1])
                        for net in root.findall("./nets/net") for n in net.findall("node"))
        expected = sorted((fp.GetReference(), p.GetNumber(), p.GetNetname())
                          for fp in board.GetFootprints() for p in fp.Pads())
        if actual != expected:
            raise RuntimeError("Schematic and PCB netlists differ")
    else:
        raise ValueError("Unsupported bridge command")


if __name__ == "__main__":
    main()
