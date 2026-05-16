use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame<'_>, area: Rect) {
    let text = "\
pad — nano-faithful editor

  Ctrl-O  Write Out (save)        Ctrl-X  Exit
  Ctrl-R  Insert file             Ctrl-K  Cut line
  Ctrl-U  Uncut (paste)           Ctrl-W  Where Is (search)
  Alt-R   Replace                 Ctrl-_  Goto line
  Alt-U   Undo                    Alt-E   Redo
  Ctrl-G  Help (this screen)      Ctrl-C  Cursor position
  Ctrl-Z  Suspend to shell

Collaboration:
  Alt-S   Share / Unshare         Alt-A   Authors overlay
  Alt-C   Copy share URL          Alt-Q   Re-show QR

Press any key to dismiss.";
    let p = Paragraph::new(text).block(Block::default().title(" Help ").borders(Borders::ALL));
    frame.render_widget(p, area);
}
