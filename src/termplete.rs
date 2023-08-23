use {
    crossterm::{
        cursor::SetCursorStyle,
        event::{DisableBracketedPaste, KeyCode, KeyModifiers},
        execute, Result,
    },
    nu_ansi_term::{Color, Style},
    reedline::{
        default_emacs_keybindings, default_vi_insert_keybindings, default_vi_normal_keybindings,
        ColumnarMenu, DefaultCompleter, DefaultHinter, DefaultPrompt, DefaultValidator,
        EditCommand, EditMode, Emacs, ExampleHighlighter, Keybindings, ListMenu, Reedline,
        ReedlineEvent, ReedlineMenu, Signal, Vi,
    },
    std::io::stdout,
    shlex
};

use reedline::CursorConfig;
use reedline::FileBackedHistory;

pub fn replloop(map:std::collections::HashMap<String, Box<dyn Fn(Vec<&str>)->bool>>) -> Result<()> {
    println!("Ctrl-D to quit");
    // quick command like parameter handling
    let vi_mode = matches!(std::env::args().nth(1), Some(x) if x == "--vi");


    //let history = Box::new(FileBackedHistory::with_file(50, "history.txt".into())?);
    let mut commands: Vec<String> = vec!["clear", "exit", "logout", "add","quit"].into_iter().map(Into::into).collect();
    let map_keys: Vec<String> = map.keys().cloned().collect();

    // Concatenate with the existing commands
    commands.extend(map_keys);


    let completer = Box::new(DefaultCompleter::new_with_wordlen(commands.clone(), 2));

    let cursor_config = CursorConfig {
        vi_insert: Some(SetCursorStyle::BlinkingBar),
        vi_normal: Some(SetCursorStyle::SteadyBlock),
        emacs: None,
    };

    // Setting history_per_session to true will allow the history to be isolated to the current session
    // Setting history_per_session to false will allow the history to be shared across all sessions
    let history_per_session = true;
    let mut history_session_id = if history_per_session {
        Reedline::create_history_session_id()
    } else {
        None
    };

    let mut line_editor = Reedline::create()
        .with_history_session_id(history_session_id)
       // .with_history(history)
        .with_history_exclusion_prefix(Some(" ".to_string()))
        .with_completer(completer)
        .with_quick_completions(true)
        .with_partial_completions(true)
        .with_cursor_config(cursor_config)
        .with_highlighter(Box::new(ExampleHighlighter::new(commands)))
        .with_hinter(Box::new(
            DefaultHinter::default().with_style(Style::new().fg(Color::DarkGray)),
        ))
        .with_validator(Box::new(DefaultValidator))
        .with_ansi_colors(true);
    let res = line_editor.enable_bracketed_paste();
    let bracketed_paste_enabled = res.is_ok();
    if !bracketed_paste_enabled {
        println!("Warn: failed to enable bracketed paste mode: {res:?}");
    }

    // Adding default menus for the compiled reedline
    line_editor = line_editor
        .with_menu(ReedlineMenu::EngineCompleter(Box::new(
            ColumnarMenu::default().with_name("completion_menu"),
        )))
        .with_menu(ReedlineMenu::HistoryMenu(Box::new(
            ListMenu::default().with_name("history_menu"),
        )));

    let edit_mode: Box<dyn EditMode> = if vi_mode {
        let mut normal_keybindings = default_vi_normal_keybindings();
        let mut insert_keybindings = default_vi_insert_keybindings();

        add_menu_keybindings(&mut normal_keybindings);
        add_menu_keybindings(&mut insert_keybindings);

        add_newline_keybinding(&mut insert_keybindings);

        Box::new(Vi::new(insert_keybindings, normal_keybindings))
    } else {
        let mut keybindings = default_emacs_keybindings();
        add_menu_keybindings(&mut keybindings);
        add_newline_keybinding(&mut keybindings);

        Box::new(Emacs::new(keybindings))
    };

    line_editor = line_editor.with_edit_mode(edit_mode);

    // Adding vi as text editor
    line_editor = line_editor.with_buffer_editor("vi".into(), "nu".into());

    let prompt = DefaultPrompt::default();

    loop {
        let sig = line_editor.read_line(&prompt);

        match sig {
            Ok(Signal::CtrlD) => {
                break;
            }
            Ok(Signal::Success(buffer)) => {
                let buffer_trimmed = buffer.trim(); // Store trimmed buffer in a variable
                if buffer_trimmed == "exit" || buffer_trimmed == "logout"|| buffer_trimmed == "quit" {
                    break;
                }
                if buffer_trimmed == "clear" {
                    line_editor.clear_scrollback()?;
                    continue;
                }
                if buffer_trimmed == "history" {
                    line_editor.print_history()?;
                    continue;
                }
                if buffer_trimmed == "history session" {
                    line_editor.print_history_session()?;
                    continue;
                }
                let args: Vec<String> = shlex::split(buffer_trimmed).unwrap_or_default();
                if let Some(command) = args.first() {
                    if let Some(func) = map.get(command) {
                        if !func(args[1..].to_vec().iter().map(AsRef::as_ref).collect()){
                        continue;
                        }
                    }
                }
                if buffer_trimmed == "history sessionid" {
                    line_editor.print_history_session_id()?;
                    continue;
                }
                if buffer_trimmed == "toggle history_session" {
                    let hist_session_id = if history_session_id.is_none() {
                        // If we never created a history session ID, create one now
                        let sesh = Reedline::create_history_session_id();
                        history_session_id = sesh;
                        sesh
                    } else {
                        history_session_id
                    };
                    line_editor.toggle_history_session_matching(hist_session_id)?;
                    continue;
                }
                if buffer_trimmed == "clear-history" {
                    let hstry = Box::new(line_editor.history_mut());
                    hstry
                        .clear()
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                    continue;
                }
                // Handle other cases or unknown command
            }
            
            Ok(Signal::CtrlC) => {
                // Prompt has been cleared and should start on the next line
            }
            Err(err) => {
                println!("Error: {err:?}");
            }
        }
    }

    if bracketed_paste_enabled {
        let _ = execute!(stdout(), DisableBracketedPaste);
    }
    println!();
    Ok(())
}

fn add_menu_keybindings(keybindings: &mut Keybindings) {
    keybindings.add_binding(
        KeyModifiers::CONTROL,
        KeyCode::Char('x'),
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("history_menu".to_string()),
            ReedlineEvent::MenuPageNext,
        ]),
    );

    keybindings.add_binding(
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        KeyCode::Char('x'),
        ReedlineEvent::MenuPagePrevious,
    );

    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("completion_menu".to_string()),
            ReedlineEvent::Edit(vec![EditCommand::Complete]),
        ]),
    );

    keybindings.add_binding(
        KeyModifiers::SHIFT,
        KeyCode::BackTab,
        ReedlineEvent::MenuPrevious,
    );
}

fn add_newline_keybinding(keybindings: &mut Keybindings) {
    // This doesn't work for macOS
    keybindings.add_binding(
        KeyModifiers::ALT,
        KeyCode::Enter,
        ReedlineEvent::Edit(vec![EditCommand::InsertNewline]),
    );
}
