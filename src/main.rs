use crossbeam_channel::{select, unbounded, Receiver};
use crossterm::{
    cursor,
    event::{Event, KeyCode, KeyModifiers},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{cmp, error, io, thread, time::Duration};
use tui::{backend::CrosstermBackend, Terminal};

use gsftp::{
    app::App,
    app_utils::ActiveState,
    config::{self, AuthMethod, Config},
    draw::{self, TextStyle},
    file_transfer, sftp,
};

fn main() -> Result<(), Box<dyn error::Error>> {
    // Command line arguments
    let args = config::args();
    // Setup static immutable Config
    let conf = Config::from(&args);

    // SFTP session
    println!("Connecting to client...");
    let sess = match &conf.auth_method {
        AuthMethod::Password(pwd) => sftp::get_session_with_password(pwd, &conf),
        AuthMethod::PrivateKey(sk) => sftp::get_session_with_pubkey_file(sk, &conf),
        AuthMethod::Agent => sftp::get_session_with_user_auth_agent(&conf),
        AuthMethod::Manual => unimplemented!(),
    }
    .unwrap_or_else(|e| {
        eprintln!("Error establishing SSH session: {e}");
        std::process::exit(1);
    });

    // Setup static mutable App
    let mut app = App::from(&sess, args);

    // Cleanup & close the Alternate Screen before logging error messages
    std::panic::set_hook(Box::new(|panic_info| {
        cleanup_terminal().unwrap();
        eprintln!("Application error: {panic_info}");
    }));

    // Initializing backend, terminal, & receivers before we attempt to establish a session
    setup_terminal()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).unwrap_or_else(|e| {
        eprintln!("Failed to create terminal: {e}");
        std::process::exit(1);
    });
    // let ticker = tick(Duration::from_secs_f64(1.0 / 60.0));
    let ui_events_receiver = setup_ui_events();
    let ctrl_c_events = setup_ctrl_c();

    draw::text_alert(
        &mut terminal,
        &mut app,
        Some("Press '?' to toggle help"),
        None,
    );

    loop {
        select! {
            recv(ctrl_c_events) -> _ => {
                break;
            }
            recv(ui_events_receiver) -> message => {
                if let Event::Key(key_event) = message.unwrap() {
                    if key_event.modifiers.is_empty() {
                        match key_event.code {
                            // quit
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            // Show/hide help
                            KeyCode::Char('?') => app.show_help = !app.show_help,
                            // toggle hidden files
                            KeyCode::Char('a') => {
                                app.show_hidden = !app.show_hidden;
                                app.content.update_local(&app.buf.local, app.show_hidden);
                                app.content.update_remote(&sess, &app.buf.remote, app.show_hidden);
                            }
                            // down
                            KeyCode::Char('j') | KeyCode::Down => match app.state.active {
                                ActiveState::Local => {
                                    // the continue prevents the function from breaking in empty dirs
                                    if app.content.local.is_empty() { continue }
                                    let curr = app.state.local.selected().unwrap();
                                    let next = cmp::min(curr + 1, app.content.local.len() - 1);
                                    app.state.local.select(Some(next));
                                },
                                ActiveState::Remote => {
                                    // the continue prevents the function from breaking in empty dirs
                                    if app.content.remote.is_empty() { continue }
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
                            // page up
                            KeyCode::Char('g') => match app.state.active {
                                ActiveState::Local =>  app.state.local.select(Some(0)),
                                ActiveState::Remote =>  app.state.remote.select(Some(0)),
                            },
                            // page down
                            // TODO: Get Vim keys 'G' to work for this
                            KeyCode::Char('b') => match app.state.active {
                                ActiveState::Local => {
                                    let i = app.content.local.len() - 1;
                                    app.state.local.select(Some(i));
                                },
                                ActiveState::Remote => {
                                    let i = app.content.remote.len() - 1;
                                    app.state.remote.select(Some(i));
                                },
                            },
                            // switch tabs
                            KeyCode::Tab  | KeyCode::Char('w') => {
                                app.state.active = match app.state.active {
                                    ActiveState::Local => ActiveState::Remote,
                                    ActiveState::Remote => ActiveState::Local,
                                }
                            },
                            // navigate into child directory
                            KeyCode::Char('l') | KeyCode::Right => match app.state.active {
                                ActiveState::Local => app.cd_into_local(),
                                ActiveState::Remote => app.cd_into_remote(&sess),
                            },
                            // navigate into parent directory (out of local directory)
                            KeyCode::Char('h') | KeyCode::Left => match app.state.active {
                                ActiveState::Local => app.cd_out_of_local(),
                                ActiveState::Remote => app.cd_out_of_remote(&sess),
                            },
                            // file transfer
                            KeyCode::Enter | KeyCode::Char('y') => match app.state.active {
                                // download
                                ActiveState::Local => {
                                    draw::text_alert(
                                        &mut terminal,
                                        &mut app,
                                        Some("Uploading..."),
                                        Some(TextStyle::text_alert())
                                    );
                                    if let Err(e) = file_transfer::upload(&sess, &app) {
                                        let err = format!("Upload error: {}", e);
                                        draw::text_alert(
                                            &mut terminal,
                                            &mut app,
                                            Some(&err),
                                            Some(TextStyle::error_message())
                                        );
                                        thread::sleep(Duration::from_millis(1800));
                                    }
                                    app.content.update_remote(&sess, &app.buf.remote, app.show_hidden);
                                },
                                // upload
                                ActiveState::Remote => {
                                    draw::text_alert(
                                        &mut terminal,
                                        &mut app,
                                        Some("Downloading..."),
                                        Some(TextStyle::text_alert())
                                    );
                                    if let Err(e) = file_transfer::download(&sess, &app) {
                                        let err = format!("download error: {}", e);
                                        draw::text_alert(
                                            &mut terminal,
                                            &mut app,
                                            Some(&err),
                                            Some(TextStyle::error_message())
                                        );
                                        thread::sleep(Duration::from_millis(1800));
                                    }
                                    app.content.update_local(&app.buf.local, app.show_hidden);
                                },
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
                            // page up
                            KeyCode::Up => match app.state.active {
                                ActiveState::Local =>  app.state.local.select(Some(0)),
                                ActiveState::Remote =>  app.state.remote.select(Some(0)),
                            },
                            // page down
                            KeyCode::Down => match app.state.active {
                                ActiveState::Local => {
                                    let i = app.content.local.len() - 1;
                                    app.state.local.select(Some(i));
                                },
                                ActiveState::Remote => {
                                    let i = app.content.remote.len() - 1;
                                    app.state.remote.select(Some(i));
                                },
                            },
                            _ => {}
                        }
                    }
                }
                draw::ui(&mut terminal, &mut app);
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
    execute!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        terminal::SetTitle("gsftp")
    )?;

    terminal::enable_raw_mode()?;

    Ok(())
}

fn cleanup_terminal() -> Result<(), io::Error> {
    let mut stdout = io::stdout();
    // TTYs don't actually have an alternate screen, so you need to
    //  clear the screen in this case.
    // We have to execute this *before* leaving the alternate screen so that
    //  the main screen is cleared iff we're running in a TTY.
    execute!(
        stdout,
        cursor::MoveTo(0, 0),
        terminal::Clear(terminal::ClearType::All)
    )?;
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

fn setup_ctrl_c() -> Receiver<()> {
    let (tx, rx) = unbounded();
    ctrlc::set_handler(move || {
        tx.send(()).unwrap();
    })
    .unwrap();

    rx
}
