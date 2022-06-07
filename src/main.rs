use std::{io, time::Duration, thread};
use tui::{backend::CrosstermBackend, Terminal};
use crossbeam_channel::{select, tick, unbounded, Receiver};
use crossterm::{
    execute, cursor,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    event::{Event, KeyCode, KeyModifiers},
};

use file_manager::{
    app::App,
    config::{args, Config, AuthMethod}, 
    draw::{draw, startup_text}, 
    readdir::DirBuf,
    tcp, 
};

fn main() -> Result<(), io::Error> {
    let conf = Config::from(args());
    
    // Cleanup & close the Alternate Screen before logging error messages
    std::panic::set_hook(Box::new(|panic_info| {
        cleanup_terminal().unwrap();
        eprintln!("Application error: {panic_info}");
    }));

    // Initializing backend, terminal, & receivers before we attempt to establish a sesion
    setup_terminal()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let ticker = tick(Duration::from_secs_f64(1.0 / 60.0));
    let ui_events_receiver = setup_ui_events();
    let ctrl_c_events = setup_ctrl_c();
    startup_text(&mut terminal);

    // SFTP session
    let mut sess = match &conf.auth_method {
        AuthMethod::Password(pwd) => tcp::get_session_with_password(pwd, &conf),
        AuthMethod::PrivateKey(_id) => tcp::get_session_with_pubkey_file(&conf),
        AuthMethod::Agent => tcp::get_session_with_userauth_agent(&conf),
    }
    .unwrap_or_else(|e| {
        cleanup_terminal().unwrap();
        eprintln!("Error establishing SSH session: {e}");
        std::process::exit(1);
    });

    let app = App::from(DirBuf::from(&mut sess), &sess);

    draw(&mut terminal, &app, &conf);

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
                                KeyCode::Char('q') => {
                                    break
                                },
                                _ => {}
                            }
                        } else if key_event.modifiers == KeyModifiers::CONTROL {
                            match key_event.code {
                                KeyCode::Char('c') => {
                                    break
                                },
                                _ => {}
                            }
                        }
                    },
                    Event::Resize(_w, _h) => {
                        draw(&mut terminal, &app, &conf);
                    },
                    _ => {}
                }
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