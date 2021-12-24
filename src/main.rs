use async_timer::{new_sync_timer, SyncTimer, Timer};
use core::time;
use crossterm::{
    cursor::{self},
    event::{poll, read, Event, KeyCode, KeyModifiers},
    style::{Color, ResetColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand, Result,
};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
    sync::mpsc::channel,
    time::Duration,
};

const input: &str = 
"large|is|paper|got|idea|tree|should|might|read|long|still|river|own|as|hand|why|study|back|same|seem|almost|leave|where|kind|say|book|song|mile|us|when|work|how|mile|hard|move|set|every|write|away|her|had|soon|on|grow|from|do|change|world|any|came|talk|below|who|until|last|up|even|look|second|need|well|close|may|quick|family|my|play|more|every|help|there|who|may|found|face|give|little|which|us|now|it's|with|call|time|water|something|many|together|right|could|form|end|see|over|big|still|did|need|also|start|keep|after|carry|some|in|part|side|far|quite|until|while|seem|sometimes|can|earth|might|in|go|had|quickly|his|same|world|our|eat|could|eat|has|with|with|under|place|saw|last|quick|way|don't|very|then|than|much|that|begin|page|second|list|do|said|the|are|around|important|than|no|put|open|was|something|day|so|add|close|life|begin|but|young|young|we|by|name|car|is|be|mountain|live|light|talk|head|ask|night|little|small|America|both|follow|this|place|who|seem|family|boy|he|cut|your|boy|sentence|almost|four|few|air|here|both|then|walk|make|their|the|right|watch|way|eye|their|list|he|again|plant|always|use|run|picture|here|want|read|year|school|went|him|only|back|all|land|another|story|when|of|story|thought|stop|idea|said|write|we|really|would|much|make|out|was|sometimes|always|water|came|were|head|group|side|tree|being|out|more|let|there|without|turn|find|soon|let|sound|hard|earth|much|off|in|even|home|man|while|food|this|through|we|first|time|know|idea|carry|why|it's|sometimes|talk|enough|well|some|for|same|from|new|made|after|state|often|different|has|quickly|oil|watch|because|food|not|tell|home|at|call|also|eye|above|him|and|been|are|when|mean|face|and|left|that|up|light|book|being|good|form|around|set|took|play|people|later|name|only|about|look|begin|point|way|never|head|city|life|girl|later|walk|next|they|want|no|once|far|side|hand|set|country|like|three|great|air|me|have|high|does|number|an|find|another|day|man|mother|run|song|Indian|found|world|did|want|got|between";

fn words_from_file(filename: impl AsRef<Path>) -> std::io::Result<String> {
    // Ok(BufReader::new(File::open(filename)?)
    //     .lines()
    //     .nth(0)
    //     .expect("Unabel to read file")?)
    Ok(String::from(input))
}

fn print_events() -> Result<()> {
    // Read words from file and randomly arange them
    let lines = words_from_file("words.txt")?;
    let mut words: Vec<&str> = lines.split("|").collect();
    words.shuffle(&mut thread_rng());

    // Initialize raw output
    let mut stdout = std::io::stdout();
    stdout.execute(Clear(ClearType::All))?;
    stdout.execute(cursor::MoveTo(0, 0))?;

    // Display current set of words
    let mut line = 0;
    for (i, word) in words.iter().enumerate() {
        print!("{} ", word);
        if i % 15 == 14 {
            if line == 1 {
                break;
            }
            println!();
            line += 1;
        }
    }
    stdout.flush()?;
    stdout.execute(cursor::MoveToColumn(0))?;
    stdout.execute(cursor::MoveUp(1))?;
    line = 0;

    let mut word_index = (0, 0); // (Word Index, Char index)
    let mut wrong_words = HashSet::new();
    let mut keystrokes = (0, 0); // (Total, wrong)
    let mut strokes = Vec::with_capacity(15);

    let mut work = new_sync_timer(time::Duration::from_secs(60));
    work.init(|_| {});

    loop {
        if poll(Duration::from_millis(100))? {
            let event = read()?;

            match event {
                Event::Key(c) => match c.code {
                    KeyCode::Char(chr) => {
                        if c.modifiers.contains(KeyModifiers::CONTROL) {
                            // Exit on ctrl-c
                            return Ok(());
                        } else if chr == ' ' {
                            // Invalidate rest of the word
                            for i in word_index.1..words[word_index.0].len() {
                                stdout.execute(SetForegroundColor(Color::Red))?;
                                stdout
                                    .write(&[words[word_index.0].chars().nth(i).unwrap() as u8])?;
                                wrong_words.insert(word_index.0);
                            }
                            stdout.execute(ResetColor)?;

                            if strokes.iter().collect::<String>() != words[word_index.0] {
                                wrong_words.insert(word_index.0);
                            }

                            // Next word, rest char index
                            word_index.0 += 1;
                            word_index.1 = 0;

                            // Handles the end of the current line was reached
                            if word_index.0 % 15 == 0 && word_index.0 != 0 {
                                stdout.execute(Clear(ClearType::All))?;
                                stdout.execute(cursor::MoveTo(0, 0))?;

                                // Printout next set of words
                                for (i, word) in words.iter().enumerate().skip(word_index.0) {
                                    print!("{} ", word);
                                    if i % 15 == 14 {
                                        if line == 1 {
                                            break;
                                        }
                                        println!();
                                        line += 1;
                                    }
                                }
                                line = 0;
                                stdout.flush()?;

                                stdout.execute(cursor::MoveTo(0, 0))?;
                            } else {
                                // Move cursor to next word
                                stdout.execute(cursor::MoveRight(1))?;
                            }
                            strokes.clear();
                        } else {
                            strokes.push(chr);
                            // Retrieve the next character in word array
                            let correct = words[word_index.0].chars().nth(word_index.1);
                            if let Some(correct_char) = correct {
                                if correct_char == chr {
                                    // If the correct character was typed
                                    stdout.execute(SetForegroundColor(Color::Green))?;
                                    stdout.write(&[chr as u8])?;
                                    stdout.execute(ResetColor)?;
                                } else {
                                    // If the incorrect character was typed
                                    stdout.execute(SetForegroundColor(Color::Red))?;
                                    stdout.write(&[correct_char as u8])?;
                                    stdout.execute(ResetColor)?;
                                    keystrokes.1 += 1;
                                }
                                // Go to next character
                                word_index.1 += 1;
                            }
                            keystrokes.0 += 1;
                        }
                    }
                    KeyCode::Backspace => {
                        // Makes sure we aren't going to a previous word
                        if word_index.1 > 0 {
                            word_index.1 -= 1;
                            // Retrieve the character that was just backspaced
                            let correct = words[word_index.0].chars().nth(word_index.1);
                            if let Some(correct_char) = correct {
                                // Reset the color of the backspaced character
                                stdout.execute(cursor::MoveLeft(1))?;
                                stdout.execute(ResetColor)?;
                                stdout.write(&[correct_char as u8])?;
                                stdout.execute(cursor::MoveLeft(1))?;
                                stdout.flush()?;
                            }
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
        } else {
            if work.is_expired() {
                break;
            }
        }
    }

    stdout.execute(cursor::MoveToColumn(0))?;
    stdout.execute(cursor::MoveDown(3))?;

    println!("WPM: {}", word_index.0 - 1);
    println!("Correct Words: {}", word_index.0 - wrong_words.len());
    println!("Incorrect Words: {}", wrong_words.len());
    print!("Keystrokes: (",);
    stdout.execute(SetForegroundColor(Color::Green))?;
    print!("{}", keystrokes.0 - keystrokes.1);
    stdout.execute(ResetColor)?;
    print!("|");
    stdout.execute(SetForegroundColor(Color::Red))?;
    print!("{}", keystrokes.1);
    stdout.execute(ResetColor)?;
    print!(") {}", keystrokes.0);
    Ok(())
}

fn main() -> Result<()> {
    enable_raw_mode()?;

    if let Err(e) = print_events() {
        println!("Error: {:?}\r", e);
    }

    disable_raw_mode()?;
    loop {}
}
