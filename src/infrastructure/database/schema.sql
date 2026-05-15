-- NexusTask Enterprise Database Schema
-- Optimized for SQLite with JSON and Nested Set support

PRAGMA foreign_keys = ON;

-- Full-Text Search for Tasks
CREATE VIRTUAL TABLE IF NOT EXISTS tasks_fts USING fts5(
    title,
    description,
    content='tasks',
    content_rowid='id'
);

-- Triggers to keep FTS in sync
CREATE TRIGGER IF NOT EXISTS tasks_fts_insert AFTER INSERT ON tasks BEGIN
    INSERT INTO tasks_fts(rowid, title, description)
    VALUES (new.id, new.title, new.description);
END;

CREATE TRIGGER IF NOT EXISTS tasks_fts_delete AFTER DELETE ON tasks BEGIN
    DELETE FROM tasks_fts WHERE rowid = old.id;
END;

CREATE TRIGGER IF NOT EXISTS tasks_fts_update AFTER UPDATE ON tasks BEGIN
    UPDATE tasks_fts SET title = new.title, description = new.description
    WHERE rowid = new.id;
END;

-- 1. Workspaces (Multi-team isolation)
CREATE TABLE IF NOT EXISTS workspaces (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT UNIQUE NOT NULL,
    description TEXT,
    owner_id TEXT NOT NULL,
    parent_workspace_id TEXT REFERENCES workspaces(id),
    lft INTEGER,
    rgt INTEGER,
    depth INTEGER DEFAULT 0,
    settings TEXT NOT NULL DEFAULT '{}',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT 1
);

CREATE INDEX IF NOT EXISTS idx_workspaces_lft ON workspaces(lft, rgt);
CREATE INDEX IF NOT EXISTS idx_workspaces_slug ON workspaces(slug);

-- 2. Users
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    full_name TEXT NOT NULL,
    role TEXT CHECK(role IN ('superadmin', 'admin', 'manager', 'user')) DEFAULT 'user',
    manager_id TEXT REFERENCES users(id),
    is_active BOOLEAN DEFAULT 1,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_login DATETIME
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_manager ON users(manager_id);

-- 3. Workspace Members
CREATE TABLE IF NOT EXISTS workspace_members (
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role TEXT CHECK(role IN ('owner', 'admin', 'member', 'viewer')) DEFAULT 'member',
    joined_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (workspace_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_ws_members_user ON workspace_members(user_id);

-- 4. Tasks
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT CHECK(status IN ('pending', 'in_progress', 'done', 'blocked')) DEFAULT 'pending',
    priority INTEGER CHECK(priority BETWEEN 1 AND 5) DEFAULT 3,
    created_by TEXT NOT NULL REFERENCES users(id),
    assigned_to TEXT REFERENCES users(id),
    parent_id TEXT REFERENCES tasks(id) ON DELETE CASCADE,
    lft INTEGER,
    rgt INTEGER,
    depth INTEGER DEFAULT 0,
    data TEXT NOT NULL DEFAULT '{}',
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    due_date DATETIME,
    version INTEGER DEFAULT 1
);

CREATE INDEX IF NOT EXISTS idx_tasks_hierarchy ON tasks(workspace_id, lft, rgt);
CREATE INDEX IF NOT EXISTS idx_tasks_parent ON tasks(parent_id);
CREATE INDEX IF NOT EXISTS idx_tasks_assigned ON tasks(assigned_to);
CREATE INDEX IF NOT EXISTS idx_tasks_workspace ON tasks(workspace_id);

-- 5. Comments
CREATE TABLE IF NOT EXISTS comments (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id),
    parent_id TEXT REFERENCES comments(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    data TEXT NOT NULL DEFAULT '{}',
    lft INTEGER,
    rgt INTEGER,
    depth INTEGER DEFAULT 0,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_comments_task_tree ON comments(task_id, lft, rgt);

-- 6. Event Log
CREATE TABLE IF NOT EXISTS event_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL REFERENCES users(id),
    event_type TEXT NOT NULL,
    entity_type TEXT,
    entity_id TEXT,
    data TEXT NOT NULL DEFAULT '{}',
    workspace_id TEXT REFERENCES workspaces(id),
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_event_log_main ON event_log(workspace_id, created_at);
CREATE INDEX IF NOT EXISTS idx_event_log_entity ON event_log(entity_type, entity_id);

-- 7. Workspace Custom Fields
CREATE TABLE IF NOT EXISTS workspace_custom_fields (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    field_name TEXT NOT NULL,
    field_type TEXT CHECK(field_type IN ('text', 'number', 'date', 'boolean', 'select')),
    field_options TEXT,
    is_required BOOLEAN DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(workspace_id, field_name)
);

-- 8. User Sessions
CREATE TABLE IF NOT EXISTS user_sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token TEXT UNIQUE NOT NULL,
    expires_at DATETIME NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    ip_address TEXT,
    user_agent TEXT
);

CREATE INDEX IF NOT EXISTS idx_sessions_token ON user_sessions(token);
CREATE INDEX IF NOT EXISTS idx_sessions_user ON user_sessions(user_id);
