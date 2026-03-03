use crate::error::{TypeError, TypeErrorCode};
use crate::types::*;
use zehd_tome::Span;

// ── Type Slots ──────────────────────────────────────────────────

/// Internal slot for a type variable in the union-find.
#[derive(Debug, Clone)]
enum TypeSlot {
    /// Fresh variable — no type assigned yet.
    Unresolved,
    /// Resolved to a concrete type.
    Resolved(Type),
    /// Union-find link to another variable.
    Link(TypeVar),
}

// ── Inference Context ───────────────────────────────────────────

/// Union-find based type inference engine.
///
/// Manages type variables and unification. Type variables are created during
/// inference and progressively unified (constrained) until they resolve to
/// concrete types.
pub struct InferCtx {
    slots: Vec<TypeSlot>,
}

impl InferCtx {
    pub fn new() -> Self {
        Self { slots: Vec::new() }
    }

    /// Create a fresh type variable.
    pub fn fresh_var(&mut self) -> TypeVar {
        let var = self.slots.len() as TypeVar;
        self.slots.push(TypeSlot::Unresolved);
        var
    }

    /// Create a fresh type variable wrapped in a Type.
    pub fn fresh(&mut self) -> Type {
        Type::Var(self.fresh_var())
    }

    /// Find the root of a type variable (with path compression).
    fn find(&mut self, var: TypeVar) -> TypeVar {
        if (var as usize) >= self.slots.len() {
            return var;
        }
        match self.slots[var as usize].clone() {
            TypeSlot::Link(next) => {
                let root = self.find(next);
                // Path compression
                self.slots[var as usize] = TypeSlot::Link(root);
                root
            }
            _ => var,
        }
    }

    /// Resolve a type, following type variable links.
    ///
    /// Returns the type with all top-level variables resolved. Does NOT
    /// recursively resolve nested types — use `zonk` for that.
    pub fn resolve(&mut self, ty: &Type) -> Type {
        match ty {
            Type::Var(v) => {
                if (*v as usize) >= self.slots.len() {
                    // Invalid var (e.g. resolver placeholder) — treat as unresolved.
                    return Type::Error;
                }
                let root = self.find(*v);
                match &self.slots[root as usize] {
                    TypeSlot::Resolved(resolved) => resolved.clone(),
                    _ => Type::Var(root),
                }
            }
            other => other.clone(),
        }
    }

    /// Deeply resolve a type — replace ALL type variables with their resolved
    /// types throughout the entire type tree. Unresolved variables become `Error`.
    pub fn zonk(&mut self, ty: &Type) -> Type {
        match ty {
            Type::Var(v) => {
                if (*v as usize) >= self.slots.len() {
                    return Type::Error;
                }
                let root = self.find(*v);
                match self.slots[root as usize].clone() {
                    TypeSlot::Resolved(resolved) => self.zonk(&resolved),
                    _ => Type::Error,
                }
            }
            Type::Option(inner) => Type::Option(Box::new(self.zonk(inner))),
            Type::Result(ok, err) => {
                Type::Result(Box::new(self.zonk(ok)), Box::new(self.zonk(err)))
            }
            Type::List(elem) => Type::List(Box::new(self.zonk(elem))),
            Type::Map(k, v) => Type::Map(Box::new(self.zonk(k)), Box::new(self.zonk(v))),
            Type::Struct(s) => Type::Struct(StructType {
                name: s.name.clone(),
                fields: s
                    .fields
                    .iter()
                    .map(|(n, t)| (n.clone(), self.zonk(t)))
                    .collect(),
                type_params: s.type_params.iter().map(|t| self.zonk(t)).collect(),
            }),
            Type::Enum(e) => Type::Enum(EnumType {
                name: e.name.clone(),
                variants: e
                    .variants
                    .iter()
                    .map(|v| EnumVariantType {
                        name: v.name.clone(),
                        payload: v.payload.as_ref().map(|t| self.zonk(t)),
                    })
                    .collect(),
                type_params: e.type_params.iter().map(|t| self.zonk(t)).collect(),
            }),
            Type::Function(f) => Type::Function(FunctionType {
                params: f.params.iter().map(|t| self.zonk(t)).collect(),
                return_type: Box::new(self.zonk(&f.return_type)),
            }),
            other => other.clone(),
        }
    }

    /// Unify two types — make them equal.
    ///
    /// On success, returns the unified type. On failure, returns a TypeError.
    /// The `span` is used for error reporting if unification fails.
    pub fn unify(&mut self, a: &Type, b: &Type, span: Span) -> Result<Type, TypeError> {
        let a = self.resolve(a);
        let b = self.resolve(b);

        match (&a, &b) {
            // Error types unify with anything — prevents cascading.
            (Type::Error, _) => Ok(Type::Error),
            (_, Type::Error) => Ok(Type::Error),

            // Never unifies with anything — diverging code.
            (Type::Never, other) | (other, Type::Never) => Ok(other.clone()),

            // Two variables — union them.
            (Type::Var(va), Type::Var(vb)) => {
                let ra = self.find(*va);
                let rb = self.find(*vb);
                if ra != rb {
                    self.slots[ra as usize] = TypeSlot::Link(rb);
                }
                Ok(Type::Var(rb))
            }

            // Variable + concrete — resolve the variable.
            (Type::Var(v), concrete) | (concrete, Type::Var(v)) => {
                let root = self.find(*v);
                // Occurs check: prevent infinite types
                if self.occurs_in(root, concrete) {
                    return Err(TypeError::error(
                        TypeErrorCode::T110,
                        "infinite type detected",
                        span,
                    )
                    .build());
                }
                self.slots[root as usize] = TypeSlot::Resolved(concrete.clone());
                Ok(concrete.clone())
            }

            // Primitives.
            (Type::Int, Type::Int) => Ok(Type::Int),
            (Type::Float, Type::Float) => Ok(Type::Float),
            (Type::String, Type::String) => Ok(Type::String),
            (Type::Bool, Type::Bool) => Ok(Type::Bool),
            (Type::Unit, Type::Unit) => Ok(Type::Unit),

            // Time unifies with Int.
            (Type::Time, Type::Time) => Ok(Type::Time),
            (Type::Time, Type::Int) | (Type::Int, Type::Time) => Ok(Type::Int),

            // Constructed types.
            (Type::Option(a_inner), Type::Option(b_inner)) => {
                let inner = self.unify(a_inner, b_inner, span)?;
                Ok(Type::Option(Box::new(inner)))
            }
            (Type::Result(a_ok, a_err), Type::Result(b_ok, b_err)) => {
                let ok = self.unify(a_ok, b_ok, span)?;
                let err = self.unify(a_err, b_err, span)?;
                Ok(Type::Result(Box::new(ok), Box::new(err)))
            }
            (Type::List(a_elem), Type::List(b_elem)) => {
                let elem = self.unify(a_elem, b_elem, span)?;
                Ok(Type::List(Box::new(elem)))
            }
            (Type::Map(ak, av), Type::Map(bk, bv)) => {
                let k = self.unify(ak, bk, span)?;
                let v = self.unify(av, bv, span)?;
                Ok(Type::Map(Box::new(k), Box::new(v)))
            }

            // Struct types — structural unification.
            (Type::Struct(sa), Type::Struct(_sb)) => {
                if Type::is_compatible(&a, &b) {
                    // Use the more specific (named) type.
                    if sa.name.is_some() {
                        Ok(a.clone())
                    } else {
                        Ok(b.clone())
                    }
                } else {
                    Err(TypeError::error(
                        TypeErrorCode::T110,
                        format!("type mismatch: expected `{b}`, found `{a}`"),
                        span,
                    )
                    .build())
                }
            }

            // Enum types — same enum.
            (Type::Enum(ea), Type::Enum(eb)) if ea.name == eb.name => Ok(a.clone()),

            // Function types.
            (Type::Function(fa), Type::Function(fb)) => {
                if fa.params.len() != fb.params.len() {
                    return Err(TypeError::error(
                        TypeErrorCode::T110,
                        format!("type mismatch: expected `{b}`, found `{a}`"),
                        span,
                    )
                    .build());
                }
                let params: Result<Vec<_>, _> = fa
                    .params
                    .iter()
                    .zip(&fb.params)
                    .map(|(pa, pb)| self.unify(pa, pb, span))
                    .collect();
                let ret = self.unify(&fa.return_type, &fb.return_type, span)?;
                Ok(Type::Function(FunctionType {
                    params: params?,
                    return_type: Box::new(ret),
                }))
            }

            _ => Err(TypeError::error(
                TypeErrorCode::T110,
                format!("type mismatch: expected `{b}`, found `{a}`"),
                span,
            )
            .build()),
        }
    }

    /// Occurs check: does the type variable `var` appear in `ty`?
    fn occurs_in(&mut self, var: TypeVar, ty: &Type) -> bool {
        match ty {
            Type::Var(v) => {
                let root = self.find(*v);
                if root == var {
                    return true;
                }
                match self.slots[root as usize].clone() {
                    TypeSlot::Resolved(resolved) => self.occurs_in(var, &resolved),
                    _ => false,
                }
            }
            Type::Option(inner) => self.occurs_in(var, inner),
            Type::Result(ok, err) => self.occurs_in(var, ok) || self.occurs_in(var, err),
            Type::List(elem) => self.occurs_in(var, elem),
            Type::Map(k, v) => self.occurs_in(var, k) || self.occurs_in(var, v),
            Type::Function(f) => {
                f.params.iter().any(|p| self.occurs_in(var, p))
                    || self.occurs_in(var, &f.return_type)
            }
            Type::Struct(s) => s.fields.iter().any(|(_, t)| self.occurs_in(var, t)),
            _ => false,
        }
    }
}

impl Default for InferCtx {
    fn default() -> Self {
        Self::new()
    }
}
