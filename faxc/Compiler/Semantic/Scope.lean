/-
Semantic Analysis - Scope Management
-/

import Compiler.Semantic.Types

namespace Compiler.Semantic.Scope

open Compiler.Semantic.Types

-- Scope stack operations
structure ScopeManager where
  scopes : List Scope
  nextLevel : Nat
  deriving Repr

def ScopeManager.new : ScopeManager :=
  { scopes := [Scope.global], nextLevel := 1 }

def ScopeManager.current (manager : ScopeManager) : Scope :=
  manager.scopes.head!

def ScopeManager.push (manager : ScopeManager) (kind : ScopeKind) : ScopeManager :=
  let newScope := {
    kind := kind
    variables := []
    functions := []
    types := []
    level := manager.nextLevel
  }
  { 
    scopes := newScope :: manager.scopes
    nextLevel := manager.nextLevel + 1
  }

def ScopeManager.pop (manager : ScopeManager) : ScopeManager :=
  match manager.scopes with
  | [] => manager
  | [_] => manager  -- Don't pop global scope
  | _ :: rest => { manager with scopes := rest }

def ScopeManager.addVariable (manager : ScopeManager) 
    (name : String) (ty : Ty) (mutable : Bool) : ScopeManager :=
  let current := manager.current
  let newCurrent := current.addVariable name ty mutable
  { manager with scopes := [newCurrent] ++ manager.scopes.tail! }

def ScopeManager.addFunction (manager : ScopeManager)
    (name : String) (params : List Ty) (ret : Ty) (pub : Bool) : ScopeManager :=
  let current := manager.current
  let newCurrent := current.addFunction name params ret pub
  { manager with scopes := [newCurrent] ++ manager.scopes.tail! }

def ScopeManager.addType (manager : ScopeManager)
    (name : String) (ty : Ty) : ScopeManager :=
  let current := manager.current
  let newCurrent := current.addType name ty
  { manager with scopes := [newCurrent] ++ manager.scopes.tail! }

def ScopeManager.lookupVariable (manager : ScopeManager) (name : String) : Option Ty :=
  manager.scopes.findSome? (λ scope => scope.lookupVariable name)

def ScopeManager.lookupFunction (manager : ScopeManager) (name : String) : Option (List Ty × Ty) :=
  manager.scopes.findSome? (λ scope => scope.lookupFunction name)

def ScopeManager.lookupType (manager : ScopeManager) (name : String) : Option Ty :=
  manager.scopes.findSome? (λ scope => scope.lookupType name)

def ScopeManager.isGlobal (manager : ScopeManager) : Bool :=
  manager.scopes.length == 1

def ScopeManager.currentLevel (manager : ScopeManager) : Nat :=
  manager.current.level

-- Check if variable is in current scope
def ScopeManager.isDefinedInCurrentScope (manager : ScopeManager) (name : String) : Bool :=
  manager.current.lookupVariable name |>.isSome

-- Get all visible variables
def ScopeManager.allVariables (manager : ScopeManager) : List (String × Ty × Bool) :=
  manager.scopes.foldr (λ scope acc => scope.variables ++ acc) []

-- Get all visible functions
def ScopeManager.allFunctions (manager : ScopeManager) : List (String × List Ty × Ty × Bool) :=
  manager.scopes.foldr (λ scope acc => scope.functions ++ acc) []

-- Shadowing check
def ScopeManager.wouldShadow (manager : ScopeManager) (name : String) : Bool :=
  match manager.scopes with
  | current :: parent :: _ =>
    current.lookupVariable name |>.isNone &&
    parent.lookupVariable name |>.isSome
  | _ => false

end Compiler.Semantic.Scope
