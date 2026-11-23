// Kept apart from archive.ts so that callers do not pull in zip.js.

export function isZipFile(file: { mime: string; filename: string }) {
  return (
    file.mime === "application/zip" ||
    file.mime === "application/x-zip-compressed" ||
    file.filename.toLowerCase().endsWith(".zip")
  );
}

export function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB"];
  let value = bytes / 1024;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit++;
  }
  return `${value.toFixed(value < 10 ? 1 : 0)} ${units[unit]}`;
}
