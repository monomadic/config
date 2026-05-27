use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, BeginSynchronizedUpdate, EndSynchronizedUpdate,
        EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

pub(crate) struct TerminalSession {
    raw_enabled: bool,
    screen_enabled: bool,
    synchronized_update: bool,
    alternate_screen_enabled: bool,
    mouse_capture_enabled: bool,
}

pub(crate) fn cleanup_terminal_state<F, G>(
    screen_enabled: &mut bool,
    raw_enabled: &mut bool,
    mut leave_screen: F,
    mut disable_raw: G,
) -> Result<()>
where
    F: FnMut() -> Result<()>,
    G: FnMut() -> Result<()>,
{
    let mut error = None;

    if *screen_enabled {
        if let Err(err) = leave_screen() {
            error = Some(err);
        }
        *screen_enabled = false;
    }

    if *raw_enabled {
        if let Err(err) = disable_raw() {
            if error.is_none() {
                error = Some(err);
            }
        }
        *raw_enabled = false;
    }

    if let Some(err) = error {
        Err(err)
    } else {
        Ok(())
    }
}

impl TerminalSession {
    pub(crate) fn enter(stdout: &mut io::Stdout) -> Result<Self> {
        enable_raw_mode()?;
        let mut session = Self {
            raw_enabled: true,
            screen_enabled: false,
            synchronized_update: false,
            alternate_screen_enabled: false,
            mouse_capture_enabled: false,
        };
        execute!(stdout, BeginSynchronizedUpdate)?;
        session.synchronized_update = true;
        execute!(stdout, EnterAlternateScreen)?;
        session.screen_enabled = true;
        session.alternate_screen_enabled = true;
        execute!(stdout, EnableMouseCapture)?;
        session.mouse_capture_enabled = true;
        Ok(session)
    }

    pub(crate) fn finish_initial_draw(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<()> {
        if self.synchronized_update {
            execute!(terminal.backend_mut(), EndSynchronizedUpdate)?;
            self.synchronized_update = false;
        }
        Ok(())
    }

    pub(crate) fn restore(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<()> {
        if self.synchronized_update {
            execute!(terminal.backend_mut(), EndSynchronizedUpdate)?;
            self.synchronized_update = false;
        }
        if self.mouse_capture_enabled {
            execute!(terminal.backend_mut(), DisableMouseCapture)?;
            self.mouse_capture_enabled = false;
        }
        let alternate_screen_enabled = self.alternate_screen_enabled;
        cleanup_terminal_state(
            &mut self.screen_enabled,
            &mut self.raw_enabled,
            || {
                if alternate_screen_enabled {
                    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                }
                Ok(())
            },
            || {
                disable_raw_mode()?;
                Ok(())
            },
        )?;
        terminal.show_cursor()?;
        Ok(())
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        if self.synchronized_update {
            let mut stdout = io::stdout();
            let _ = execute!(stdout, EndSynchronizedUpdate);
            self.synchronized_update = false;
        }
        if self.mouse_capture_enabled {
            let mut stdout = io::stdout();
            let _ = execute!(stdout, DisableMouseCapture);
            self.mouse_capture_enabled = false;
        }
        let alternate_screen_enabled = self.alternate_screen_enabled;
        let _ = cleanup_terminal_state(
            &mut self.screen_enabled,
            &mut self.raw_enabled,
            || {
                let mut stdout = io::stdout();
                if alternate_screen_enabled {
                    execute!(stdout, LeaveAlternateScreen)?;
                }
                Ok(())
            },
            || {
                disable_raw_mode()?;
                Ok(())
            },
        );
    }
}

pub(crate) fn finish_with_restore(
    run_result: Result<()>,
    restore_result: Result<()>,
) -> Result<()> {
    match (run_result, restore_result) {
        (Err(run_err), Err(restore_err)) => {
            Err(run_err.context(format!("terminal restore also failed: {restore_err}")))
        }
        (Err(run_err), Ok(())) => Err(run_err),
        (Ok(()), Err(restore_err)) => Err(restore_err),
        (Ok(()), Ok(())) => Ok(()),
    }
}
