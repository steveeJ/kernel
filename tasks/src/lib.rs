#![no_std]
#![feature(asm)]

extern crate x86;
extern crate interrupts;
extern crate spin;

use spin::Mutex;

pub mod stack {
    use core;
    use core::ops::Range;

    pub struct Stack {
        pub bottom: usize,
        pub top: usize,
    }

    impl Stack {
        pub fn contains(&self, t: usize) -> bool {
            self.bottom <= t && t <= self.top
        }
    }

    trait IsStack {
        fn is_initialized(&self) -> bool;
    }

    impl IsStack for Stack {
        fn is_initialized(&self) -> bool {
            unimplemented!()
        }
    }


    impl core::fmt::LowerHex for Stack {
        fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
            write!(f, "0x{:x}..0x{:x}", self.bottom, self.top)
        }
    }
}

pub type TaskEntrySlice = [TaskEntry; 3];

pub struct TaskStateInformation {
    pub current_task: usize,
    pub next_task: usize,
    pub tasks: TaskEntrySlice,
}

impl TaskStateInformation {
    pub fn new(tasks: TaskEntrySlice) -> TaskStateInformation {
        TaskStateInformation {
            current_task: 0,
            next_task: 0,
            tasks: tasks,
        }
    }

    /// Choose the next task.
    /// Return true if tasks will be switched
    pub fn schedule_next(&mut self) -> bool {
        self.next_task = self.current_task;
        while {
                  self.next_task = (self.next_task + 1) % self.tasks.len();
                  self.next_task != self.current_task && self.tasks[self.next_task].blocked == true
              } {}
        self.next_task != self.current_task
    }

    pub fn get_current_task(&self) -> &TaskEntry {
        &self.tasks[self.current_task]
    }

    pub fn get_current_task_mut(&mut self) -> &mut TaskEntry {
        &mut self.tasks[self.current_task]
    }


    /// Update the esf of the current_task
    /// Returns the new esf that can be used by the ISR
    pub fn mangle_esf_for_next(&mut self,
                               esf: &interrupts::ExceptionStackFrame)
                               -> &interrupts::ExceptionStackFrame {
        // let alligned_stack_pointer = (esf.stack_pointer + 0x10 - 1) & !(0x10 - 1);
        // assert_eq!(esf.stack_pointer, alligned_stack_pointer);

        {
            let old_esf = &mut self.tasks[self.current_task].esf;
            old_esf.instruction_pointer = esf.instruction_pointer;
            old_esf.stack_pointer = esf.stack_pointer;
            old_esf.cpu_flags = esf.cpu_flags;
        }

        let next_esf = &mut self.tasks[self.next_task].esf;

        // next_esf.instruction_pointer = if next_esf.instruction_pointer ==
        //                                   TASK_ENTRY_UNITIALIZED_USIZE {
        //     esf.instruction_pointer
        // } else {
        //     next_esf.instruction_pointer
        // };

        // next_esf.stack_pointer = if next_esf.stack_pointer == TASK_ENTRY_UNITIALIZED_USIZE {
        //     esf.stack_pointer
        // } else {
        //     next_esf.stack_pointer
        // };

        // next_esf.instruction_pointer = if next_esf.instruction_pointer ==
        //                                   TASK_ENTRY_UNITIALIZED_USIZE {
        //     esf.instruction_pointer
        // } else {
        //     next_esf.instruction_pointer
        // };

        // next_esf.cpu_flags = if next_esf.cpu_flags == TASK_ENTRY_UNITIALIZED_U64 {
        //     esf.cpu_flags
        // } else {
        //     next_esf.cpu_flags
        // };

        // TODO: move this to a place where the task has *really* been switched
        self.current_task = self.next_task;

        next_esf
    }
}

pub struct TaskEntry {
    pub name: &'static str,
    pub esf: interrupts::ExceptionStackFrame,
    pub stack: stack::Stack,
    pub registers: TaskRegisters,
    pub blocked: bool,
}

impl TaskEntry {}

#[derive(Clone,Copy)]
#[repr(C,packed)]
pub struct TaskRegisters {
    pub rax: usize,
    pub rbx: usize,
    pub rcx: usize,
    pub rdx: usize,
    pub rsi: usize,
    pub rdi: usize,
    pub r8: usize,
    pub r9: usize,
    pub r10: usize,
    pub r11: usize,
    pub r12: usize,
    pub r13: usize,
    pub r14: usize,
    pub r15: usize,
    pub rbp: usize,
}

impl TaskRegisters {
    pub fn empty() -> TaskRegisters {
        TaskRegisters {
            rbp: 0,
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            r11: 0,
            r10: 0,
            r8: 0,
            r9: 0,
            rdi: 0,
            rsi: 0,
            rdx: 0,
            rcx: 0,
            rbx: 0,
            rax: 0,
        }
    }
}

impl core::fmt::Display for TaskRegisters {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f,
               "rax: {} rbx: {} rcx: {} rdx: {} rsi: {} rdi: {} r8:  {} r9:  {} r10: {} r11: {} r12: {} r13: {} r14: {} r15: {} rbp: {}",
               self.rax,
               self.rbx,
               self.rcx,
               self.rdx,
               self.rsi,
               self.rdi,
               self.r8,
               self.r9,
               self.r10,
               self.r11,
               self.r12,
               self.r13,
               self.r14,
               self.r15,
               self.rbp)
    }
}

impl core::fmt::LowerHex for TaskRegisters {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f,
               "rax: 0x{:x} rbx: 0x{:x} rcx: 0x{:x} rdx: 0x{:x} rsi: 0x{:x} rdi: 0x{:x} r8:  0x{:x} r9:  0x{:x} r10: 0x{:x} r11: 0x{:x} r12: 0x{:x} r13: 0x{:x} r14: 0x{:x} r15: 0x{:x} rbp: 0x{:x}",
               self.rax,
               self.rbx,
               self.rcx,
               self.rdx,
               self.rsi,
               self.rdi,
               self.r8,
               self.r9,
               self.r10,
               self.r11,
               self.r12,
               self.r13,
               self.r14,
               self.r15,
               self.rbp)
    }
}

pub fn initilaze_tss() {
    // TODO: create a valid TSS structure

    // TODO: find the TSS descriptor memory address:
    let sdgt: usize;
    unsafe {
        asm!("

            "
            // output operands
            :
            // input operands
            :
            // clobbers
            :
            // options
            : "intel");
    }

    // TODO: write the TSS structure at the memory address load the TSS

    /* TODO
    ; load TSS
    mov ax, gdt64.tss ; relative index of tss in gdt64
    ltr ax
    */
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
