pub trait Renderer<OutputChar, OutputLine> {
    fn render_char(character: char) -> OutputChar;
    fn render_line(line: &[OutputChar]) -> OutputLine;
}
