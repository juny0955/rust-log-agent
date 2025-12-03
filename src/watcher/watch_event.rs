pub enum WatchEvent {
    NewLine(String),
    Rotated,
    EndOfFile,
}