use std::{
    any::Any,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use iced::{
    alignment,
    widget::{button, column, container, radio, row, text},
    Border, Theme,
};

use super::{Message, Window};
use crate::core::{CpuMode, InstructionMode, RegisterBank};

pub struct DebuggerWindow {
    registers: RegisterBank,
    emulation_running: Arc<AtomicBool>,
}

impl DebuggerWindow {
    pub fn new(emulation_running: Arc<AtomicBool>) -> Self {
        Self {
            registers: RegisterBank::default(),
            emulation_running,
        }
    }

    pub fn update_values(&mut self, registers: RegisterBank) {
        self.registers = registers;
    }

    fn emulation_controls(&self) -> iced::Element<Message> {
        let is_running = self.emulation_running.load(Ordering::SeqCst);
        let run_label = if is_running { "Pause" } else { "Run" };
        let step_message = if is_running {
            None
        } else {
            Some(Message::StepEmulation)
        };
        let reset_message = if is_running {
            None
        } else {
            Some(Message::ResetEmulation)
        };

        row![
            button(run_label).on_press(Message::ChangeRunningState),
            button("Step").on_press_maybe(step_message),
            button("Reset").on_press_maybe(reset_message),
        ]
        .spacing(10)
        .padding(10)
        .into()
    }

    fn registers_view(&self) -> iced::Element<Message> {
        titled_view(
            "Registers",
            column![
                row![
                    field_value("R0", self.registers.reg(0)),
                    field_value("R1", self.registers.reg(1)),
                    field_value("R2", self.registers.reg(2)),
                    field_value("R3", self.registers.reg(3)),
                ],
                row![
                    field_value("R4", self.registers.reg(4)),
                    field_value("R5", self.registers.reg(5)),
                    field_value("R6", self.registers.reg(6)),
                    field_value("R7", self.registers.reg(7)),
                ],
                row![
                    field_value("R8", self.registers.reg(8)),
                    field_value("R9", self.registers.reg(9)),
                    field_value("R10", self.registers.reg(10)),
                    field_value("R11", self.registers.reg(11)),
                ],
                row![
                    field_value("R12", self.registers.reg(12)),
                    field_value("R13 (SP)", self.registers.reg(13)),
                    field_value("R14 (LR)", self.registers.reg(14)),
                    field_value("R15 (PC)", self.registers.reg(15))
                ],
            ]
            .into(),
        )
        .into()
    }

    fn instruction_mode_view(&self) -> iced::Element<Message> {
        titled_view(
            "Instruction Mode",
            row![
                radio(
                    "Arm",
                    InstructionMode::Arm,
                    Some(self.registers.cpsr.instruction_mode),
                    Message::SetInstructionMode
                ),
                radio(
                    "Thumb",
                    InstructionMode::Thumb,
                    Some(self.registers.cpsr.instruction_mode),
                    Message::SetInstructionMode
                ),
            ]
            .spacing(10)
            .padding(10)
            .into(),
        )
        .into()
    }

    fn cpu_mode_view(&self) -> iced::Element<Message> {
        titled_view(
            "CPU Mode",
            row![
                radio(
                    "User",
                    CpuMode::User,
                    Some(self.registers.cpsr.mode),
                    Message::SetCpuMode
                ),
                radio(
                    "FIQ",
                    CpuMode::Fiq,
                    Some(self.registers.cpsr.mode),
                    Message::SetCpuMode
                ),
                radio(
                    "IRQ",
                    CpuMode::Irq,
                    Some(self.registers.cpsr.mode),
                    Message::SetCpuMode
                ),
                radio(
                    "Supervisor",
                    CpuMode::Supervisor,
                    Some(self.registers.cpsr.mode),
                    Message::SetCpuMode
                ),
                radio(
                    "Abort",
                    CpuMode::Abort,
                    Some(self.registers.cpsr.mode),
                    Message::SetCpuMode
                ),
                radio(
                    "System",
                    CpuMode::System,
                    Some(self.registers.cpsr.mode),
                    Message::SetCpuMode
                )
            ]
            .spacing(10)
            .padding(10)
            .into(),
        )
        .into()
    }
}

fn titled_view<'a>(
    label: &'a str,
    content: iced::Element<'a, Message>,
) -> iced::Element<'a, Message> {
    column![
        title_label(label),
        container(content).style(|theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style {
                border: Border {
                    width: 2.0,
                    radius: 5.0.into(),
                    color: palette.background.weak.color,
                },
                ..container::Style::default()
            }
        })
    ]
    .spacing(2)
    .into()
}

fn title_label(label: &str) -> iced::Element<Message> {
    text(label)
        .width(iced::Length::Fill)
        .align_x(iced::Alignment::Start)
        .align_y(alignment::Vertical::Center)
        .size(12)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            text::Style {
                color: Some(palette.background.weak.color),
            }
        })
        .into()
}

fn field_value(name: &str, value: u32) -> iced::Element<Message> {
    row![
        text(name)
            .align_y(alignment::Vertical::Center)
            .width(iced::Length::FillPortion(1))
            .height(iced::Length::Fill),
        container(
            text(format!("0x{:08X}", value))
                .align_x(iced::Alignment::End)
                .align_y(alignment::Vertical::Center)
                .height(iced::Length::Fill)
                .width(iced::Length::Fill)
        )
        .padding(8)
        .width(iced::Length::FillPortion(2))
        .height(iced::Length::Fill)
        .style(container::rounded_box)
    ]
    .height(50)
    .padding(8)
    .into()
}

impl Window for DebuggerWindow {
    fn title(&self) -> String {
        "Debugger".to_string()
    }

    fn view(&self) -> iced::Element<Message> {
        column![
            self.emulation_controls(),
            row![self.instruction_mode_view(), self.cpu_mode_view()],
            self.registers_view()
        ]
        .padding(8)
        .spacing(8)
        .into()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
