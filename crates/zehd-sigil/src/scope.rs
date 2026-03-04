use std::collections::HashMap;

use zehd_tome::Span;

use crate::types::Type;

// ── Identifiers ─────────────────────────────────────────────────

pub type ScopeId = u32;

// ── Scope Arena ─────────────────────────────────────────────────

/// Arena-allocated scope tree. Scopes reference parents by ScopeId.
pub struct ScopeArena {
    scopes: Vec<Scope>,
}

impl ScopeArena {
    pub fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    /// Create a new scope with the given kind and optional parent.
    pub fn create(&mut self, kind: ScopeKind, parent: Option<ScopeId>) -> ScopeId {
        let id = self.scopes.len() as ScopeId;
        self.scopes.push(Scope {
            kind,
            parent,
            symbols: HashMap::new(),
        });
        id
    }

    /// Get a reference to a scope.
    pub fn get(&self, id: ScopeId) -> &Scope {
        &self.scopes[id as usize]
    }

    /// Get a mutable reference to a scope.
    pub fn get_mut(&mut self, id: ScopeId) -> &mut Scope {
        &mut self.scopes[id as usize]
    }

    /// Define a symbol in the given scope. Returns `false` if already defined.
    pub fn define(&mut self, scope_id: ScopeId, name: String, symbol: Symbol) -> bool {
        let scope = self.get_mut(scope_id);
        if scope.symbols.contains_key(&name) {
            return false;
        }
        scope.symbols.insert(name, symbol);
        true
    }

    /// Define or update a symbol in the given scope (insert or overwrite).
    pub fn upsert(&mut self, scope_id: ScopeId, name: String, symbol: Symbol) {
        let scope = self.get_mut(scope_id);
        scope.symbols.insert(name, symbol);
    }

    /// Look up a symbol by name, walking the parent chain.
    pub fn lookup(&self, scope_id: ScopeId, name: &str) -> Option<(ScopeId, &Symbol)> {
        let scope = self.get(scope_id);
        if let Some(sym) = scope.symbols.get(name) {
            return Some((scope_id, sym));
        }
        if let Some(parent) = scope.parent {
            return self.lookup(parent, name);
        }
        None
    }

    /// Mutable lookup — find a symbol and return a mutable reference.
    pub fn lookup_mut(&mut self, scope_id: ScopeId, name: &str) -> Option<&mut Symbol> {
        // First, find which scope contains the symbol.
        let target_scope = {
            let mut current = scope_id;
            loop {
                let scope = self.get(current);
                if scope.symbols.contains_key(name) {
                    break Some(current);
                }
                match scope.parent {
                    Some(parent) => current = parent,
                    None => break None,
                }
            }
        };
        if let Some(sid) = target_scope {
            self.get_mut(sid).symbols.get_mut(name)
        } else {
            None
        }
    }

    /// Mark a symbol as used. Returns true if the symbol was found.
    pub fn mark_used(&mut self, scope_id: ScopeId, name: &str) -> bool {
        let scope = self.get_mut(scope_id);
        if let Some(sym) = scope.symbols.get_mut(name) {
            sym.used = true;
            return true;
        }
        if let Some(parent) = scope.parent {
            return self.mark_used(parent, name);
        }
        false
    }

    /// Check if the scope or any ancestor is a Loop scope.
    pub fn is_in_loop(&self, scope_id: ScopeId) -> bool {
        let scope = self.get(scope_id);
        if scope.kind == ScopeKind::Loop {
            return true;
        }
        if let Some(parent) = scope.parent {
            return self.is_in_loop(parent);
        }
        false
    }

    /// Check if the scope or any ancestor is an HttpHandler, Init, or ErrorHandler scope.
    pub fn is_in_handler(&self, scope_id: ScopeId) -> bool {
        let scope = self.get(scope_id);
        match scope.kind {
            ScopeKind::HttpHandler | ScopeKind::Init | ScopeKind::ErrorHandler => true,
            _ => {
                if let Some(parent) = scope.parent {
                    self.is_in_handler(parent)
                } else {
                    false
                }
            }
        }
    }

    /// Find the nearest enclosing Function or HttpHandler scope, returning its id.
    pub fn enclosing_function(&self, scope_id: ScopeId) -> Option<ScopeId> {
        let scope = self.get(scope_id);
        match scope.kind {
            ScopeKind::Function | ScopeKind::HttpHandler => Some(scope_id),
            _ => {
                if let Some(parent) = scope.parent {
                    self.enclosing_function(parent)
                } else {
                    None
                }
            }
        }
    }

    /// Iterate over all symbols in a scope (non-recursive).
    pub fn symbols(&self, scope_id: ScopeId) -> impl Iterator<Item = (&String, &Symbol)> {
        self.get(scope_id).symbols.iter()
    }
}

impl Default for ScopeArena {
    fn default() -> Self {
        Self::new()
    }
}

// ── Scope ───────────────────────────────────────────────────────

pub struct Scope {
    pub kind: ScopeKind,
    pub parent: Option<ScopeId>,
    pub symbols: HashMap<String, Symbol>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    /// Top-level file scope.
    Module,
    /// Server-scope declarations (top-level in route files).
    Server,
    /// Inside get/post/put/patch/delete blocks.
    HttpHandler,
    /// Inside init { }.
    Init,
    /// Inside error(e) { }.
    ErrorHandler,
    /// Inside fn or arrow function.
    Function,
    /// Bare { } or if/match branches.
    Block,
    /// for/while — enables break/continue validation.
    Loop,
}

// ── Symbol ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Symbol {
    pub kind: SymbolKind,
    pub ty: Type,
    pub mutable: bool,
    pub defined_at: Span,
    pub used: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Variable,
    Function,
    TypeDef,
    EnumDef,
    EnumVariant,
    Parameter,
    Import,
}
