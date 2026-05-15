use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame<'_>, area: Rect) {
    let text = "\
pad — nano-faithful editor

  ^O  Write Out (save)        ^X  Exit
  ^R  Insert file             ^K  Cut line
  ^U  Uncut (paste)           ^W  Where Is (search)
  M-R Replace                 ^_  Goto line
  M-U Undo                    M-E Redo
  ^G  Help (this screen)      ^C  Cursor position
  ^Z  Suspend to shell

Press any key to dismiss.";
    let p = Paragraph::new(text).block(Block::default().title(" Help ").borders(Borders::ALL));
    frame.render_widget(p, area);
}
