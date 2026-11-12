// Read-only native geometry check. Inspect only the requested app process.
import AppKit
import CoreGraphics
import Foundation

guard CommandLine.arguments.count == 2,
      let pid = Int32(CommandLine.arguments[1]) else {
    fputs("Usage: swift scripts/check-panel.swift <Kroneum process ID>\n", stderr)
    exit(2)
}

guard let windows = CGWindowListCopyWindowInfo(.optionOnScreenOnly, kCGNullWindowID) as? [[String: Any]],
      let primary = NSScreen.screens.first else {
    fputs("Native window or screen metadata is unavailable.\n", stderr)
    exit(2)
}

let panels = windows.compactMap { info -> CGRect? in
    guard (info[kCGWindowOwnerPID as String] as? NSNumber)?.int32Value == pid,
          let bounds = info[kCGWindowBounds as String] as? NSDictionary,
          let rect = CGRect(dictionaryRepresentation: bounds) else { return nil }
    // Exclude status items and tiny helper windows without reading window titles.
    return rect.width >= 300 && rect.height >= 300 ? rect : nil
}

guard panels.count == 1, let panel = panels.first else {
    fputs("Expected one visible Kroneum panel. Open the panel before this check.\n", stderr)
    exit(2)
}

// WindowServer uses top-left coordinates. AppKit uses bottom-left coordinates.
let visibleFrames = NSScreen.screens.map { screen -> CGRect in
    let frame = screen.visibleFrame
    return CGRect(x: frame.minX, y: primary.frame.maxY - frame.maxY,
                  width: frame.width, height: frame.height)
}

guard let index = visibleFrames.firstIndex(where: { $0.insetBy(dx: -1, dy: -1).contains(panel) }) else {
    fputs("The panel does not fit within any screen's visible frame.\n", stderr)
    print("Panel: \(panel). Visible screen frames: \(visibleFrames).")
    exit(1)
}

print("Native placement passed: panel \(panel) fits screen \(index + 1) of \(visibleFrames.count).")
