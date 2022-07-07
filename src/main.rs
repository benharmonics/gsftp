use crossbeam_channel::{bounded, select, tick, unbounded, Receiver};
use crossterm::{
    cursor,
    event::{Event, KeyCode, KeyModifiers},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    cmp, error, io,
    thread::{self, JoinHandle},
    time::Duration,
};
use tui::{backend::CrosstermBackend, Terminal};

use gsftp::{
    app::App,
    app_utils::ActiveState,
    config::{self, AuthMethod, Config},
    draw::UiWindow,
    file_transfer::Transfer,
    sftp,
};

fn main() -> Result<(), Box<dyn error::Error>> {
    // Command line arguments
    let args = config::args();
    // Set up static immutable Config
    let conf = Config::from(&args);
    // SSH session
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
    // Establish SFTP connection via SSH
    let sftp = sess.sftp()?;
    // Setup static mutable App
    let mut app = App::from(&sess, &sftp, args);
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
    // variables related to our tick receiver
    const FPS: f64 = 60.0;
    let mut ticks_elapsed: u8 = 0;
    // receivers
    let ui_events_receiver = setup_ui_events();
    let ctrl_c_events = setup_ctrl_c();
    let ticker = tick(Duration::from_secs_f64(1.0 / FPS));
    // vector to store our thread handles
    let mut handles: Vec<JoinHandle<()>> = vec![];
    // vector to store receivers from threads
    let mut receivers: Vec<Receiver<String>> = vec![];
    // User Interface struct
    let mut window = UiWindow::default();
    let mut user_has_pressed_buttons = false;
    let mut completed_transfers = 0;

    loop {
        // block until action occurs
        select! {
            recv(ctrl_c_events) -> _ => {
                break;
            }
            recv(ticker) -> _ => {
                // Check for updates once every second (at 60 fps)
                ticks_elapsed = (ticks_elapsed + 1) % FPS as u8;
                if ticks_elapsed == 0 {
                    app.content.update_local(&app.buf.local, app.show_hidden);
                    app.content.update_remote(&sftp, &app.buf.remote, app.show_hidden);
                    // Reset window periodically when there's no info to show
                    if user_has_pressed_buttons && receivers.len() == completed_transfers {
                        window.reset();
                    }
                    // Check if any of our receivers completed
                    for receiver in &receivers {
                        match receiver.try_recv() {
                            Ok(message) => if message.is_empty() {
                                completed_transfers += 1;
                            } else {
                                window.error_message(message.as_str());
                                completed_transfers += 1;
                            },
                            Err(_) => {},
                        }
                    }
                }
                window.draw(&mut terminal, &mut app);
            }
            recv(ui_events_receiver) -> message => {
                if let Event::Key(key_event) = message.unwrap() {
                    user_has_pressed_buttons = true;
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
                                app.content.update_remote(&sftp, &app.buf.remote, app.show_hidden);
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
                            KeyCode::Char('g') | KeyCode::Char('t') => match app.state.active {
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
                                ActiveState::Remote => app.cd_into_remote(&sftp),
                            },
                            // navigate into parent directory (out of local directory)
                            KeyCode::Char('h') | KeyCode::Left => match app.state.active {
                                ActiveState::Local => app.cd_out_of_local(),
                                ActiveState::Remote => app.cd_out_of_remote(&sftp),
                            },
                            // file transfer
                            KeyCode::Enter | KeyCode::Char('y') => match app.state.active {
                                // upload
                                ActiveState::Local => {
                                    window.flashing_text("Uploading...");
                                    let transfer = Transfer::upload(&app, &sess);
                                    spawn_transfer_thread(transfer, &mut handles, &mut receivers);
                                    app.content.update_remote(&sftp, &app.buf.remote, app.show_hidden);
                                },
                                // download
                                ActiveState::Remote => {
                                    window.flashing_text("Downloading...");
                                    let transfer = Transfer::download(&app, &sess);
                                    spawn_transfer_thread(transfer, &mut handles, &mut receivers);
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
            }
        }
    }

    cleanup_terminal()?;

    for handle in handles {
        handle.join().unwrap();
    }

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

fn spawn_transfer_thread(
    transfer: Transfer,
    handles: &mut Vec<JoinHandle<()>>,
    receivers: &mut Vec<Receiver<String>>,
) {
    let (tx, rx) = bounded(1);
    handles.push(thread::spawn(move || {
        tx.send(match transfer.execute() {
            Ok(_) => String::new(),
            Err(err) => format!("Transfer error: {}", err),
        })
        .unwrap();
    }));
    receivers.push(rx);
}
