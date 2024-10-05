use std::collections::HashMap;

use super::{
    value::Value,
    vm::{Vm, VmError},
};

pub enum ScopeSearch {
    CurrentOnly,
    AllScopes,
}

pub struct Local {
    pub pinned: bool,
    pub value: Value,
}

impl Local {
    pub fn set(&mut self, value: Value) {
        self.value = value;
    }
}

pub struct Scope {
    locals: HashMap<String, Local>,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            locals: HashMap::new(),
        }
    }

    pub fn get_local(&self, name: &String) -> Option<&Local> {
        self.locals.get(name)
    }

    pub fn get_local_mut(&mut self, name: &String) -> Option<&mut Local> {
        self.locals.get_mut(name)
    }
}

impl Vm {
    pub fn begin_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn end_scope(&mut self) {
        if self.scopes.is_empty() {
            panic!("scope stack underflow");
        }
        self.scopes.pop();
    }

    pub fn set_local(&mut self, name: &String, value: Value) -> Result<(), VmError> {
        if let Some(local) = self.get_local_mut(name, ScopeSearch::AllScopes) {
            if local.pinned {
                Err(VmError::PinnedLocal)
            } else {
                local.set(value);
                Ok(())
            }
        } else {
            Err(VmError::UndefinedLocal)
        }
    }

    pub fn get_local_mut(&mut self, name: &String, search: ScopeSearch) -> Option<&mut Local> {
        match search {
            ScopeSearch::CurrentOnly => {
                let last = self.scopes.last_mut().unwrap();
                last.get_local_mut(name)
            }
            ScopeSearch::AllScopes => {
                for s in self.scopes.iter_mut().rev() {
                    match s.get_local_mut(name) {
                        Some(l) => return Some(l),
                        _ => (),
                    };
                }
                return None;
            }
        }
    }

    pub fn get_local(&self, name: &String, search: ScopeSearch) -> Option<&Local> {
        match search {
            ScopeSearch::CurrentOnly => {
                let last = self.scopes.last().unwrap();
                last.get_local(name)
            }
            ScopeSearch::AllScopes => {
                for s in self.scopes.iter().rev() {
                    match s.get_local(name) {
                        Some(l) => return Some(l),
                        _ => (),
                    };
                }
                return None;
            }
        }
    }

    pub fn define_local(
        &mut self,
        name: String,
        value: Value,
        pinned: bool,
    ) -> Result<(), VmError> {
        let last_scope = self.scopes.last_mut().ok_or(VmError::InvalidOperation)?;
        let existing_local = last_scope.get_local_mut(&name);

        match existing_local {
            Some(Local {
                value: _,
                pinned: true,
            }) => Err(VmError::PinnedLocal),
            Some(local) => {
                local.pinned = true;
                Ok(())
            }
            None => {
                last_scope.locals.insert(name, Local { value, pinned });
                Ok(())
            }
        }
    }
}
