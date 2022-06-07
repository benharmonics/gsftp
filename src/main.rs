use std::{io, time::Duration, thread};
use tui::{backend::CrosstermBackend, Terminal};
use crossterm::{
    execute, cursor,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use file_manager::{
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

    setup_terminal()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    startup_text(&mut terminal);

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

    let directories = DirBuf::from(&mut sess);

    draw(&mut terminal, &directories, &mut sess, &conf);

    thread::sleep(Duration::from_millis(5000));

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