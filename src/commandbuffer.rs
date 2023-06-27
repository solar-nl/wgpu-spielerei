use std::collections::VecDeque;

pub enum Command {
    Play,
    Pause,
    PlayForward,
    PlayReverse,
    DebugDraw,
/*
    IncreaseVolume,
    DecreaseVolume,
*/
    Quit
}

pub struct CommandBuffer {
    commands:VecDeque<Command>
}

impl CommandBuffer {
    pub fn new() -> Self {
        Self {
            commands: VecDeque::new()
        }
    }

    pub fn add_command(&mut self, command:Command) {
        self.commands.push_back(command);
    }

    pub fn next_command(&mut self) -> Option<Command> {
        self.commands.pop_front()
    }
}