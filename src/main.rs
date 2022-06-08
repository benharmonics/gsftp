use std::{cmp, io, thread};
use tui::{backend::CrosstermBackend, Terminal};
use crossbeam_channel::{select, unbounded, Receiver};
use crossterm::{
    cursor, execute, 
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    event::{Event, KeyCode, KeyModifiers},
};

use file_manager::{
    app::{ActiveState, App},
    config::{self, AuthMethod, Config}, 
    draw::{draw, startup_text}, 
    dir_utils::DirBuf,
    sftp, 
};

fn main() -> Result<(), io::Error> {
    // Command line arguments
    let args = config::args();
    // Setup static immutable Config
    let conf = Config::from(&args);
    
    // Cleanup & close the Alternate Screen before logging error messages
    std::panic::set_hook(Box::new(|panic_info| {
        cleanup_terminal().unwrap();
        eprintln!("Application error: {panic_info}");
    }));

    // Initializing backend, terminal, & receivers before we attempt to establish a sesion
    setup_terminal()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).unwrap_or_else(|e| {
        eprintln!("Failed to create terminal: {e}");
        std::process::exit(1);
    });
    // let ticker = tick(Duration::from_secs_f64(1.0 / 60.0));
    let ui_events_receiver = setup_ui_events();
    let ctrl_c_events = setup_ctrl_c();
    startup_text(&mut terminal);

    // SFTP session
    let mut sess = match &conf.auth_method {
        AuthMethod::Password(pwd) => sftp::get_session_with_password(pwd, &conf),
        AuthMethod::PrivateKey(_id) => sftp::get_session_with_pubkey_file(&conf),
        AuthMethod::Agent => sftp::get_session_with_userauth_agent(&conf),
    }
    .unwrap_or_else(|e| {
        cleanup_terminal().unwrap();
        eprintln!("Error establishing SSH session: {e}");
        std::process::exit(1);
    });

    // Setup static mutable App
    let mut app = App::from(DirBuf::from(&mut sess), &sess, args);

    draw(&mut terminal, &mut app);

    loop {
        select! {
            recv(ctrl_c_events) -> _ => {
                break;
            }
            recv(ui_events_receiver) -> message => {
                match message.unwrap() {
                    Event::Key(key_event) => {
                        if key_event.modifiers.is_empty() {
                            match key_event.code {
                                // quit
                                KeyCode::Char('q') | KeyCode::Esc => break,
                                // Show/hide help
                                KeyCode::Char('?') => app.show_help = !app.show_help,
                                // down
                                KeyCode::Char('j') | KeyCode::Down => match app.state.active {
                                    ActiveState::Local => {
                                        // the continue prevents the function from breaking for empty dirs
                                        if app.content.local.len() == 0 { continue }
                                        let curr = app.state.local.selected().unwrap();
                                        let next = cmp::min(curr + 1, app.content.local.len() - 1);
                                        app.state.local.select(Some(next));
                                    },
                                    ActiveState::Remote => {
                                        // the continue prevents the function from breaking for empty dirs
                                        if app.content.remote.len() == 0 { continue }
                                        let curr = app.state.remote.selected().unwrap();
                                        let next = cmp::min(curr + 1, app.content.remote.len() - 1);
                                        app.state.remote.select(Some(next));
                                    },
                                },
                                // up
                                KeyCode::Char('k') | KeyCode::Up => match app.state.active {
                                    ActiveState::Local => {
                                        let curr = app.state.local.selected().unwrap();
                                        let next = if curr > 0 { curr - 1 } else { curr };
                                        app.state.local.select(Some(next));
                                    },
                                    ActiveState::Remote => {
                                        let curr = app.state.remote.selected().unwrap();
                                        let next = if curr > 0 { curr - 1 } else { curr };
                                        app.state.remote.select(Some(next));
                                    },
                                },
                                // switch tabs
                                KeyCode::Tab  | KeyCode::Char('w') => {
                                    app.state.active = match app.state.active {
                                        ActiveState::Local => ActiveState::Remote,
                                        ActiveState::Remote => ActiveState::Local,
                                    }
                                },
                                KeyCode::Char('l') | KeyCode::Right => match app.state.active {
                                    ActiveState::Local => app.cd_into_local(),
                                    ActiveState::Remote => app.cd_into_remote(&sess),
                                },
                                KeyCode::Char('h') | KeyCode::Left => match app.state.active {
                                    ActiveState::Local => app.cd_out_of_local(),
                                    ActiveState::Remote => app.cd_out_of_remote(&sess),
                                },
                                _ => {}
                            }
                        } else if key_event.modifiers == KeyModifiers::CONTROL {
                            match key_event.code {
                                // quit
                                KeyCode::Char('c') => break,
                                // switch tabs
                                KeyCode::Char('w') => app.state.active = match app.state.active {
                                    ActiveState::Local => ActiveState::Remote,
                                    ActiveState::Remote => ActiveState::Local,
                                },
                                _ => {}
                            }
                        }
                    },
                    _ => {}
                }
                draw(&mut terminal, &mut app);
            }
        }
    }

    cleanup_terminal()?;

    Ok(())
}

fn setup_terminal() -> Result<(), io::Error> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    // TTYs don't actually have an alternate screen, so you need to
    //  clear the screen in this case.
    // We have to execute this *after* entering the alternate screen so that
    //  the main screen is cleared iff we're running in a TTY.
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

    terminal::enable_raw_mode()?;
    terminal::SetTitle("gsftp");

    Ok(())
}

fn cleanup_terminal() -> Result<(), io::Error> {
    let mut stdout = io::stdout();
    // TTYs don't actually have an alternate screen, so you need to
    //  clear the screen in this case.
    // We have to execute this *before* leaving the alternate screen so that
    //  the main screen is cleared iff we're running in a TTY.
    execute!(stdout, cursor::MoveTo(0, 0), terminal::Clear(terminal::ClearType::All))?;
    execute!(stdout, LeaveAlternateScreen, cursor::Show)?;

    terminal::disable_raw_mode()?;

    Ok(())
}

// TODO: Figure out how to handle these unwraps in the tx.send(...unwrap()).unwrap()
fn setup_ui_events() -> Receiver<Event> {
    let (tx, rx) = unbounded();
    thread::spawn(move || loop {
        tx.send(crossterm::event::read().unwrap()).unwrap()
    });

    rx
}

fn setup_ctrl_c() -> Receiver <()> {
    let (tx, rx) = unbounded();
    ctrlc::set_handler(move || {
        tx.send(()).unwrap();
    })
    .unwrap();

    rx
}