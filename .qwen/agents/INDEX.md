# Agent System - Index

Comprehensive multi-agent system with strict quality controls and specialized roles.

## Quick Reference

### Core Agents

| Agent | Purpose | When to Use |
|-------|---------|-------------|
| 🎯 **orchestrator** | Task coordination | Always - entry point for all tasks |
| ✅ **quality-controller** | Quality enforcement | Before any merge, critical code review |
| 🔍 **code-reviewer** | Code review | Every PR, before merge |
| 💻 **software-engineer** | General coding | Feature implementation, bug fixes |
| 🔒 **security-engineer** | Security review | Authentication, sensitive data, APIs |
| 🐛 **bug-hunter-pro** | Bug detection | Vulnerability scanning, debugging |

### Implementation Agents

| Agent | Purpose | When to Use |
|-------|---------|-------------|
| 🏗️ **architect-engineer** | System design | New systems, major refactors |
| ⚙️ **backend-engineer** | Backend development | APIs, databases, server logic |
| 🎨 **frontend-engineer** | Frontend development | UI components, web apps |
| 📱 **mobile-engineer** | Mobile development | iOS, Android apps |
| 🔌 **api-engineer** | API design | REST/GraphQL API development |

### Infrastructure & Data Agents

| Agent | Purpose | When to Use |
|-------|---------|-------------|
| 🚀 **devops-engineer** | Infrastructure | CI/CD, deployment, monitoring |
| 🗄️ **database-engineer** | Database design | Schema, queries, optimization |
| 📊 **data-engineer** | Data pipelines | ETL, data warehousing |
| 🏗️ **platform-engineer** | Developer platform | Internal tools, DX |

### AI & ML Agents

| Agent | Purpose | When to Use |
|-------|---------|-------------|
| 🤖 **ml-engineer** | Machine learning | ML models, AI features |
| 🧠 **ai-engineer** | AI integration | AI-powered tools, code completion |
| 📝 **nlp-engineer** | NLP | Text analysis, documentation generation |
| 🔄 **mlops-engineer** | MLOps | Model deployment, monitoring |
| 👁️ **computer-vision-engineer** | Computer Vision | Image processing, visualization |
| 🧩 **knowledge-engineer** | Knowledge Systems | Ontology, expert systems, knowledge graphs |

### Quality & Testing Agents

| Agent | Purpose | When to Use |
|-------|---------|-------------|
| 🧪 **test-engineer** | Testing | Test strategy, implementation |
| 📋 **qa-engineer** | Quality assurance | Test planning, QA processes |
| ⚡ **performance-engineer** | Performance | Optimization, profiling |
| 🛡️ **reliability-engineer** | SRE | Reliability, monitoring, incidents |

### Design & Documentation Agents

| Agent | Purpose | When to Use |
|-------|---------|-------------|
| 📚 **documentation-writer** | Documentation | READMEs, API docs, guides |
| 🎭 **ux-engineer** | User experience | UX design, accessibility |

### Fax Compiler Sub-Agents

| Agent | Purpose | When to Use |
|-------|---------|-------------|
| 🔧 **fax-compiler-agent** | Fax Compiler | Compiler development, all phases |
| 📜 **fax-lexer-agent** | Lexer (faxc-lex) | Lexical analysis, tokenization |

## Agent Categories

### Specialist Agents (Implementation)
- `software-engineer` - General purpose coding
- `frontend-engineer` - Web UI development
- `backend-engineer` - Server-side development
- `mobile-engineer` - Mobile app development
- `api-engineer` - API design and development

### Architecture & Design
- `architect-engineer` - System architecture
- `ux-engineer` - User experience design
- `platform-engineer` - Platform architecture

### Quality & Testing
- `quality-controller` - Final quality gate
- `code-reviewer` - Code review
- `test-engineer` - Test implementation
- `qa-engineer` - QA strategy
- `bug-hunter-pro` - Bug detection

### Security & Reliability
- `security-engineer` - Security review
- `reliability-engineer` - SRE
- `performance-engineer` - Performance optimization

### Infrastructure & Data
- `devops-engineer` - Infrastructure and CI/CD
- `database-engineer` - Database design
- `data-engineer` - Data pipelines
- `ml-engineer` - Machine learning

### Documentation
- `documentation-writer` - Technical writing

## Workflow

```
┌─────────────────┐
│     User        │
│    Request      │
└────────┬────────┘
         │
         ▼
┌─────────────────────────────────────────┐
│           Orchestrator (Luna)           │
│  - Analyzes request                     │
│  - Breaks into tasks                    │
│  - Selects appropriate agents           │
│  - Coordinates execution                │
│  - Reviews and integrates results       │
└─────────────────┬───────────────────────┘
                  │
         ┌────────┴────────┐
         │                 │
         ▼                 ▼
┌─────────────────┐ ┌─────────────────┐
│  Implementation │ │    Quality      │
│     Agents      │ │     Agents      │
├─────────────────┤ ├─────────────────┤
│ software-eng    │ │ code-reviewer   │
│ frontend-eng    │ │ quality-ctrl    │
│ backend-eng     │ │ security-eng    │
│ mobile-eng      │ │ bug-hunter      │
│ api-eng         │ │ test-engineer   │
│ database-eng    │ │ qa-engineer     │
│ devops-eng      │ │                 │
│ data-eng        │ │                 │
│ ml-eng          │ │                 │
└─────────────────┘ └─────────────────┘
```

## Quality Gates

Every task must pass through these quality gates:

```
Development → Code Review → Testing → Quality Control → Merge
   (1)          (2)          (3)         (4)           (5)
```

### Gate 1: Development
- Code implemented by specialist agent
- Follows best practices
- Initial tests written

### Gate 2: Code Review
- Reviewed by `code-reviewer`
- Style and conventions checked
- Logic verified

### Gate 3: Testing
- Tests implemented by `test-engineer`
- Coverage meets threshold (80%+)
- All tests passing

### Gate 4: Quality Control
- Final review by `quality-controller`
- Security checked by `security-engineer`
- Documentation verified
- **Mandatory for all merges**

### Gate 5: Merge
- Approved by orchestrator
- Ready for production

## Communication Protocol

### With User (Bahasa Indonesia)
- Hangat, profesional, lugas
- Jelaskan strategi dan hasil
- Transparan tentang kendala

### With Agents (English)
- Structured instructions
- Clear acceptance criteria
- Technical standards specified
- Examples when needed

### Artifacts (English)
- Code comments
- Documentation
- API specifications
- Commit messages

## Agent Selection Guide

### By Task Type

#### New Feature Development
```
1. architect-engineer → Design architecture
2. software-engineer → Implement feature
3. test-engineer → Write tests
4. code-reviewer → Review code
5. quality-controller → Final approval
6. documentation-writer → Update docs
```

#### Bug Fix
```
1. bug-hunter-pro → Identify root cause
2. software-engineer → Fix bug
3. test-engineer → Add regression tests
4. code-reviewer → Review fix
5. quality-controller → Approve
```

#### Security Review
```
1. security-engineer → Security audit
2. bug-hunter-pro → Vulnerability scan
3. software-engineer → Fix issues
4. quality-controller → Verify fixes
```

#### Performance Optimization
```
1. performance-engineer → Profile and analyze
2. software-engineer → Implement optimizations
3. test-engineer → Performance tests
4. reliability-engineer → Monitor impact
```

#### Infrastructure Change
```
1. devops-engineer → Design infrastructure
2. reliability-engineer → Review reliability
3. security-engineer → Security review
4. quality-controller → Final approval
```

#### Database Changes
```
1. database-engineer → Design schema
2. backend-engineer → Implement migrations
3. test-engineer → Test migrations
4. quality-controller → Approve
```

## Enforcement Rules

### Non-Negotiable
1. ❌ No merge without code review
2. ❌ No merge without tests
3. ❌ No tests with < 80% coverage on critical code
4. ❌ No security vulnerabilities
5. ❌ No hardcoded credentials
6. ❌ No undocumented breaking changes

### Quality Standards
1. ✅ Clean code principles
2. ✅ SOLID design
3. ✅ DRY (no duplication)
4. ✅ Comprehensive documentation
5. ✅ Proper error handling
6. ✅ Input validation

## File Structure

```
.qwen/agents/
├── INDEX.md                    # This file
├── orchestrator.md             # Orchestrator prompt
├── quality-controller.md       # Quality control
├── code-reviewer.md            # Code review
├── software-engineer.md        # Software development
├── security-engineer.md        # Security
├── bug-hunter-pro.md           # Bug detection
├── architect-engineer.md       # Architecture
├── backend-engineer.md         # Backend development
├── frontend-engineer.md        # Frontend development
├── database-engineer.md        # Database design
├── devops-engineer.md          # DevOps/Infrastructure
├── documentation-writer.md     # Documentation
├── reliability-engineer.md     # SRE
├── test-engineer.md            # Testing
├── qa-engineer.md              # QA
├── performance-engineer.md     # Performance
├── api-engineer.md             # API design
├── mobile-engineer.md          # Mobile development
├── data-engineer.md            # Data engineering
├── ml-engineer.md              # Machine learning
├── ux-engineer.md              # UX design
└── platform-engineer.md        # Platform engineering
```

## Version

**Version:** 1.0  
**Last Updated:** 2024  
**Maintained By:** Orchestrator (Luna)

---

*Remember: Great software is built by great teams with great processes.*
