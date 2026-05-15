use crate::domain::task::Task;
use crate::domain::user::User;
use crate::domain::workspace::{Workspace, WorkspaceMember, WorkspaceRole};
use crate::domain::comment::Comment;
use crate::domain::audit::EventLog;
use crate::ports::repository::{TaskRepository, UserRepository, WorkspaceRepository, WorkspaceMemberRepository, CommentRepository, EventLogRepository};
use rusqlite::{params, Connection, Row};
use uuid::Uuid;
use anyhow::Result;
use std::sync::{Arc, Mutex};

pub struct SqliteTaskRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteTaskRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    fn row_to_task(row: &Row) -> std::result::Result<Task, rusqlite::Error> {
        Ok(Task {
            id: Uuid::parse_str(row.get::<_, String>(0)?.as_str()).unwrap_or_else(|_| Uuid::nil()),
            title: row.get(1)?,
            description: row.get(2)?,
            status: match row.get::<_, String>(3)?.as_str() {
                "pending" => crate::domain::task::TaskStatus::Pending,
                "in_progress" => crate::domain::task::TaskStatus::InProgress,
                "done" => crate::domain::task::TaskStatus::Done,
                "blocked" => crate::domain::task::TaskStatus::Blocked,
                _ => crate::domain::task::TaskStatus::Pending,
            },
            priority: row.get(4)?,
            created_by: Uuid::parse_str(row.get::<_, String>(5)?.as_str()).unwrap_or_else(|_| Uuid::nil()),
            assigned_to: row.get::<_, Option<String>>(6)?.and_then(|s| Uuid::parse_str(&s).ok()),
            parent_id: row.get::<_, Option<String>>(7)?.and_then(|s| Uuid::parse_str(&s).ok()),
            lft: row.get(8)?,
            rgt: row.get(9)?,
            depth: row.get(10)?,
            data: serde_json::from_str(row.get::<_, String>(11)?.as_str()).unwrap_or_default(),
            workspace_id: Uuid::parse_str(row.get::<_, String>(12)?.as_str()).unwrap_or_else(|_| Uuid::nil()),
            created_at: row.get(13)?,
            updated_at: row.get(14)?,
            due_date: row.get(15)?,
            version: row.get(16)?,
        })
    }

    fn recalculate_nested_set(&self, workspace_id: Uuid) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        // Get all tasks ordered by creation
        let mut stmt = conn.prepare(
            "SELECT id, parent_id FROM tasks WHERE workspace_id = ?1 ORDER BY created_at"
        )?;
        let tasks: Vec<(String, Option<String>)> = stmt.query_map([workspace_id.to_string()], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?.collect::<Result<Vec<_>, _>>()?;

        // Build adjacency list
        let mut children: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        for (id, parent_id) in &tasks {
            let parent = parent_id.clone().unwrap_or_else(|| "root".to_string());
            children.entry(parent).or_default().push(id.clone());
        }

        // Calculate nested set using DFS
        let mut counter: i64 = 1;
        fn dfs(
            node: &str,
            depth: i32,
            counter: &mut i64,
            children: &std::collections::HashMap<String, Vec<String>>,
            conn: &Connection,
        ) -> Result<()> {
            let lft = *counter;
            *counter += 1;
            
            if let Some(child_ids) = children.get(node) {
                for child_id in child_ids {
                    dfs(child_id, depth + 1, counter, children, conn)?;
                }
            }
            
            let rgt = *counter;
            *counter += 1;
            
            if node != "root" {
                conn.execute(
                    "UPDATE tasks SET lft = ?1, rgt = ?2, depth = ?3 WHERE id = ?4",
                    params![lft, rgt, depth, node],
                )?;
            }
            
            Ok(())
        }

        dfs("root", 0, &mut counter, &children, &conn)?;
        Ok(())
    }
}

impl TaskRepository for SqliteTaskRepository {
    fn create(&self, task: Task) -> Result<Uuid> {
        let conn = self.conn.lock().unwrap();
        
        // Calculate nested set if has parent
        let (lft, rgt, depth) = if let Some(parent_id) = task.parent_id {
            let parent: Option<Task> = conn.query_row(
                "SELECT * FROM tasks WHERE id = ?1",
                [parent_id.to_string()],
                |row| SqliteTaskRepository::row_to_task(row)
            ).ok();
            
            if let Some(p) = parent {
                // Shift all nodes to the right of parent
                conn.execute(
                    "UPDATE tasks SET lft = lft + 2 WHERE lft > ?1",
                    [p.rgt],
                )?;
                conn.execute(
                    "UPDATE tasks SET rgt = rgt + 2 WHERE rgt >= ?1",
                    [p.rgt],
                )?;
                (p.rgt, p.rgt + 1, p.depth + 1)
            } else {
                (0, 1, 0)
            }
        } else {
            // Find max rgt in workspace
            let max_rgt: i64 = conn.query_row(
                "SELECT COALESCE(MAX(rgt), 0) FROM tasks WHERE workspace_id = ?1",
                [task.workspace_id.to_string()],
                |row| row.get(0)
            ).unwrap_or(0);
            (max_rgt + 1, max_rgt + 2, 0)
        };

        conn.execute(
            "INSERT INTO tasks (id, title, description, status, priority, created_by, assigned_to, parent_id, lft, rgt, depth, data, workspace_id, created_at, updated_at, due_date, version)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                task.id.to_string(),
                task.title,
                task.description,
                task.status.as_str(),
                task.priority,
                task.created_by.to_string(),
                task.assigned_to.map(|id| id.to_string()),
                task.parent_id.map(|id| id.to_string()),
                lft,
                rgt,
                depth,
                serde_json::to_string(&task.data)?,
                task.workspace_id.to_string(),
                task.created_at,
                task.updated_at,
                task.due_date,
                task.version,
            ],
        )?;
        Ok(task.id)
    }

    fn update(&self, task: Task) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE tasks SET title = ?2, description = ?3, status = ?4, priority = ?5, assigned_to = ?6, updated_at = ?7, version = version + 1 WHERE id = ?1",
            params![
                task.id.to_string(),
                task.title,
                task.description,
                task.status.as_str(),
                task.priority,
                task.assigned_to.map(|id| id.to_string()),
                chrono::Utc::now(),
            ],
        )?;
        Ok(())
    }

    fn delete(&self, id: Uuid) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM tasks WHERE id = ?1", params![id.to_string()])?;
        Ok(())
    }

    fn find_by_id(&self, id: Uuid) -> Result<Option<Task>> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT * FROM tasks WHERE id = ?1",
            [id.to_string()],
            |row| SqliteTaskRepository::row_to_task(row)
        );
        match result {
            Ok(task) => Ok(Some(task)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn get_tree(&self, root_id: Uuid) -> Result<Vec<Task>> {
        let conn = self.conn.lock().unwrap();
        let root = self.find_by_id(root_id)?;
        match root {
            Some(r) => {
                let mut stmt = conn.prepare(
                    "SELECT * FROM tasks WHERE workspace_id = ?1 AND lft BETWEEN ?2 AND ?3 ORDER BY lft"
                )?;
                let tasks = stmt.query_map(
                    [r.workspace_id.to_string(), r.lft.to_string(), r.rgt.to_string()],
                    |row| SqliteTaskRepository::row_to_task(row)
                )?.collect::<std::result::Result<Vec<_>, _>>()?;
                Ok(tasks)
            }
            None => Ok(vec![]),
        }
    }

    fn get_workspace_tasks(&self, workspace_id: Uuid) -> Result<Vec<Task>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT * FROM tasks WHERE workspace_id = ?1 ORDER BY lft"
        )?;
        let tasks = stmt.query_map(
            [workspace_id.to_string()],
            |row| SqliteTaskRepository::row_to_task(row)
        )?.collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(tasks)
    }

    fn move_task(&self, task_id: Uuid, new_parent_id: Option<Uuid>) -> Result<()> {
        let task = self.find_by_id(task_id)?;
        if task.is_none() {
            return Ok(());
        }
        let task = task.unwrap();
        
        // Remove from old position
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE tasks SET lft = lft - 2, rgt = rgt - 2 WHERE lft > ?1",
            [task.rgt.to_string()],
        )?;
        conn.execute(
            "UPDATE tasks SET rgt = rgt - 2 WHERE rgt >= ?1 AND id != ?2",
            [task.rgt.to_string(), task_id.to_string()],
        )?;

        // Add to new position
        if let Some(parent_id) = new_parent_id {
            let parent = self.find_by_id(parent_id)?;
            if let Some(p) = parent {
                conn.execute(
                    "UPDATE tasks SET lft = lft + 2 WHERE lft > ?1",
                    [p.rgt],
                )?;
                conn.execute(
                    "UPDATE tasks SET rgt = rgt + 2 WHERE rgt >= ?1",
                    [p.rgt],
                )?;
                conn.execute(
                    "UPDATE tasks SET parent_id = ?1, lft = ?2, rgt = ?3, depth = ?4 WHERE id = ?5",
                    params![parent_id.to_string(), p.rgt, p.rgt + 1, p.depth + 1, task_id.to_string()],
                )?;
            }
        } else {
            let max_rgt: i64 = conn.query_row(
                "SELECT COALESCE(MAX(rgt), 0) FROM tasks WHERE workspace_id = ?1",
                [task.workspace_id.to_string()],
                |row| row.get(0)
            ).unwrap_or(0);
            conn.execute(
                "UPDATE tasks SET parent_id = NULL, lft = ?1, rgt = ?2, depth = 0 WHERE id = ?3",
                params![max_rgt + 1, max_rgt + 2, task_id.to_string()],
            )?;
        }

        // Recalculate nested set for workspace
        self.recalculate_nested_set(task.workspace_id)?;
        Ok(())
    }

    fn get_subordinates(&self, user_id: Uuid) -> Result<Vec<User>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "WITH RECURSIVE subordinates AS (
                SELECT * FROM users WHERE manager_id = ?1
                UNION
                SELECT u.* FROM users u
                INNER JOIN subordinates s ON u.manager_id = s.id
            )
            SELECT * FROM subordinates"
        )?;
        let users = stmt.query_map(
            [user_id.to_string()],
            |row| SqliteUserRepository::row_to_user(row)
        )?.collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(users)
    }
}

pub struct SqliteUserRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteUserRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    fn row_to_user(row: &Row) -> std::result::Result<User, rusqlite::Error> {
        Ok(User {
            id: Uuid::parse_str(row.get::<_, String>(0)?.as_str()).unwrap_or_else(|_| Uuid::nil()),
            username: row.get(1)?,
            email: row.get(2)?,
            password_hash: row.get(3)?,
            full_name: row.get(4)?,
            role: match row.get::<_, String>(5)?.as_str() {
                "superadmin" => crate::domain::user::UserRole::SuperAdmin,
                "admin" => crate::domain::user::UserRole::Admin,
                "manager" => crate::domain::user::UserRole::Manager,
                "user" => crate::domain::user::UserRole::User,
                _ => crate::domain::user::UserRole::User,
            },
            manager_id: row.get::<_, Option<String>>(6)?.and_then(|s| Uuid::parse_str(&s).ok()),
            is_active: row.get(7)?,
            metadata: serde_json::from_str(row.get::<_, String>(8)?.as_str()).unwrap_or_default(),
            created_at: row.get(9)?,
            last_login: row.get(10)?,
        })
    }
}

impl UserRepository for SqliteUserRepository {
    fn create(&self, user: User) -> Result<Uuid> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO users (id, username, email, password_hash, full_name, role, manager_id, is_active, metadata, created_at, last_login)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                user.id.to_string(),
                user.username,
                user.email,
                user.password_hash,
                user.full_name,
                user.role.as_str(),
                user.manager_id.map(|id| id.to_string()),
                user.is_active,
                serde_json::to_string(&user.metadata)?,
                user.created_at,
                user.last_login,
            ],
        )?;
        Ok(user.id)
    }

    fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT * FROM users WHERE id = ?1",
            [id.to_string()],
            |row| SqliteUserRepository::row_to_user(row)
        );
        match result {
            Ok(user) => Ok(Some(user)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT * FROM users WHERE username = ?1",
            [username],
            |row| SqliteUserRepository::row_to_user(row)
        );
        match result {
            Ok(user) => Ok(Some(user)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT * FROM users WHERE email = ?1",
            [email],
            |row| SqliteUserRepository::row_to_user(row)
        );
        match result {
            Ok(user) => Ok(Some(user)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn update(&self, user: User) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE users SET username = ?2, email = ?3, full_name = ?4, role = ?5, manager_id = ?6, is_active = ?7, metadata = ?8 WHERE id = ?1",
            params![
                user.id.to_string(),
                user.username,
                user.email,
                user.full_name,
                user.role.as_str(),
                user.manager_id.map(|id| id.to_string()),
                user.is_active,
                serde_json::to_string(&user.metadata)?,
            ],
        )?;
        Ok(())
    }

    fn update_last_login(&self, user_id: Uuid) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE users SET last_login = ?1 WHERE id = ?2",
            params![chrono::Utc::now(), user_id.to_string()],
        )?;
        Ok(())
    }
}

pub struct SqliteWorkspaceRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteWorkspaceRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    fn row_to_workspace(row: &Row) -> std::result::Result<Workspace, rusqlite::Error> {
        Ok(Workspace {
            id: Uuid::parse_str(row.get::<_, String>(0)?.as_str()).unwrap_or_else(|_| Uuid::nil()),
            name: row.get(1)?,
            slug: row.get(2)?,
            description: row.get(3)?,
            owner_id: Uuid::parse_str(row.get::<_, String>(4)?.as_str()).unwrap_or_else(|_| Uuid::nil()),
            parent_workspace_id: row.get::<_, Option<String>>(5)?.and_then(|s| Uuid::parse_str(&s).ok()),
            lft: row.get(6)?,
            rgt: row.get(7)?,
            depth: row.get(8)?,
            settings: serde_json::from_str(row.get::<_, String>(9)?.as_str()).unwrap_or_default(),
            created_at: row.get(10)?,
            is_active: row.get(11)?,
        })
    }
}

impl WorkspaceRepository for SqliteWorkspaceRepository {
    fn create(&self, workspace: Workspace) -> Result<Uuid> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO workspaces (id, name, slug, description, owner_id, parent_workspace_id, lft, rgt, depth, settings, created_at, is_active)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                workspace.id.to_string(),
                workspace.name,
                workspace.slug,
                workspace.description,
                workspace.owner_id.to_string(),
                workspace.parent_workspace_id.map(|id| id.to_string()),
                workspace.lft,
                workspace.rgt,
                workspace.depth,
                serde_json::to_string(&workspace.settings)?,
                workspace.created_at,
                workspace.is_active,
            ],
        )?;
        Ok(workspace.id)
    }

    fn find_by_id(&self, id: Uuid) -> Result<Option<Workspace>> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT * FROM workspaces WHERE id = ?1",
            [id.to_string()],
            |row| SqliteWorkspaceRepository::row_to_workspace(row)
        );
        match result {
            Ok(ws) => Ok(Some(ws)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn find_by_slug(&self, slug: &str) -> Result<Option<Workspace>> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT * FROM workspaces WHERE slug = ?1",
            [slug],
            |row| SqliteWorkspaceRepository::row_to_workspace(row)
        );
        match result {
            Ok(ws) => Ok(Some(ws)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn update(&self, workspace: Workspace) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE workspaces SET name = ?2, slug = ?3, description = ?4, settings = ?5, is_active = ?6 WHERE id = ?1",
            params![
                workspace.id.to_string(),
                workspace.name,
                workspace.slug,
                workspace.description,
                serde_json::to_string(&workspace.settings)?,
                workspace.is_active,
            ],
        )?;
        Ok(())
    }

    fn get_user_workspaces(&self, user_id: Uuid) -> Result<Vec<Workspace>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT w.* FROM workspaces w
             INNER JOIN workspace_members wm ON w.id = wm.workspace_id
             WHERE wm.user_id = ?1"
        )?;
        let workspaces = stmt.query_map(
            [user_id.to_string()],
            |row| SqliteWorkspaceRepository::row_to_workspace(row)
        )?.collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(workspaces)
    }
}

pub struct SqliteWorkspaceMemberRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteWorkspaceMemberRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    fn row_to_member(row: &Row) -> std::result::Result<WorkspaceMember, rusqlite::Error> {
        Ok(WorkspaceMember {
            workspace_id: Uuid::parse_str(row.get::<_, String>(0)?.as_str()).unwrap_or_else(|_| Uuid::nil()),
            user_id: Uuid::parse_str(row.get::<_, String>(1)?.as_str()).unwrap_or_else(|_| Uuid::nil()),
            role: match row.get::<_, String>(2)?.as_str() {
                "owner" => WorkspaceRole::Owner,
                "admin" => WorkspaceRole::Admin,
                "member" => WorkspaceRole::Member,
                "viewer" => WorkspaceRole::Viewer,
                _ => WorkspaceRole::Member,
            },
            joined_at: row.get(3)?,
        })
    }
}

impl WorkspaceMemberRepository for SqliteWorkspaceMemberRepository {
    fn add_member(&self, member: WorkspaceMember) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let role_str = match member.role {
            WorkspaceRole::Owner => "owner",
            WorkspaceRole::Admin => "admin",
            WorkspaceRole::Member => "member",
            WorkspaceRole::Viewer => "viewer",
        };
        conn.execute(
            "INSERT OR REPLACE INTO workspace_members (workspace_id, user_id, role, joined_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                member.workspace_id.to_string(),
                member.user_id.to_string(),
                role_str,
                member.joined_at,
            ],
        )?;
        Ok(())
    }

    fn remove_member(&self, workspace_id: Uuid, user_id: Uuid) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM workspace_members WHERE workspace_id = ?1 AND user_id = ?2",
            params![workspace_id.to_string(), user_id.to_string()],
        )?;
        Ok(())
    }

    fn update_role(&self, workspace_id: Uuid, user_id: Uuid, role: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE workspace_members SET role = ?1 WHERE workspace_id = ?2 AND user_id = ?3",
            params![role, workspace_id.to_string(), user_id.to_string()],
        )?;
        Ok(())
    }

    fn get_members(&self, workspace_id: Uuid) -> Result<Vec<WorkspaceMember>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT * FROM workspace_members WHERE workspace_id = ?1"
        )?;
        let members = stmt.query_map(
            [workspace_id.to_string()],
            |row| SqliteWorkspaceMemberRepository::row_to_member(row)
        )?.collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(members)
    }

    fn get_user_role(&self, workspace_id: Uuid, user_id: Uuid) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT role FROM workspace_members WHERE workspace_id = ?1 AND user_id = ?2",
            params![workspace_id.to_string(), user_id.to_string()],
            |row| row.get::<_, String>(0)
        );
        match result {
            Ok(role) => Ok(Some(role)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

pub struct SqliteCommentRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteCommentRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    fn row_to_comment(row: &Row) -> std::result::Result<Comment, rusqlite::Error> {
        Ok(Comment {
            id: Uuid::parse_str(row.get::<_, String>(0)?.as_str()).unwrap_or_else(|_| Uuid::nil()),
            task_id: Uuid::parse_str(row.get::<_, String>(1)?.as_str()).unwrap_or_else(|_| Uuid::nil()),
            user_id: Uuid::parse_str(row.get::<_, String>(2)?.as_str()).unwrap_or_else(|_| Uuid::nil()),
            parent_id: row.get::<_, Option<String>>(3)?.and_then(|s| Uuid::parse_str(&s).ok()),
            content: row.get(4)?,
            data: serde_json::from_str(row.get::<_, String>(5)?.as_str()).unwrap_or_default(),
            lft: row.get(6)?,
            rgt: row.get(7)?,
            depth: row.get(8)?,
            workspace_id: Uuid::parse_str(row.get::<_, String>(9)?.as_str()).unwrap_or_else(|_| Uuid::nil()),
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    }
}

impl CommentRepository for SqliteCommentRepository {
    fn create(&self, comment: Comment) -> Result<Uuid> {
        let conn = self.conn.lock().unwrap();
        
        // Calculate nested set if has parent
        let (lft, rgt, depth) = if let Some(parent_id) = comment.parent_id {
            let parent: Option<Comment> = conn.query_row(
                "SELECT * FROM comments WHERE id = ?1",
                [parent_id.to_string()],
                |row| SqliteCommentRepository::row_to_comment(row)
            ).ok();
            
            if let Some(p) = parent {
                conn.execute(
                    "UPDATE comments SET lft = lft + 2 WHERE lft > ?1 AND task_id = ?2",
                    params![p.rgt, comment.task_id.to_string()],
                )?;
                conn.execute(
                    "UPDATE comments SET rgt = rgt + 2 WHERE rgt >= ?1 AND task_id = ?2",
                    params![p.rgt, comment.task_id.to_string()],
                )?;
                (p.rgt, p.rgt + 1, p.depth + 1)
            } else {
                (0, 1, 0)
            }
        } else {
            let max_rgt: i64 = conn.query_row(
                "SELECT COALESCE(MAX(rgt), 0) FROM comments WHERE task_id = ?1",
                [comment.task_id.to_string()],
                |row| row.get(0)
            ).unwrap_or(0);
            (max_rgt + 1, max_rgt + 2, 0)
        };

        conn.execute(
            "INSERT INTO comments (id, task_id, user_id, parent_id, content, data, lft, rgt, depth, workspace_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                comment.id.to_string(),
                comment.task_id.to_string(),
                comment.user_id.to_string(),
                comment.parent_id.map(|id| id.to_string()),
                comment.content,
                serde_json::to_string(&comment.data)?,
                lft,
                rgt,
                depth,
                comment.workspace_id.to_string(),
                comment.created_at,
                comment.updated_at,
            ],
        )?;
        Ok(comment.id)
    }

    fn update(&self, comment: Comment) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE comments SET content = ?2, data = ?3, updated_at = ?4 WHERE id = ?1",
            params![
                comment.id.to_string(),
                comment.content,
                serde_json::to_string(&comment.data)?,
                chrono::Utc::now(),
            ],
        )?;
        Ok(())
    }

    fn delete(&self, id: Uuid) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM comments WHERE id = ?1", params![id.to_string()])?;
        Ok(())
    }

    fn find_by_id(&self, id: Uuid) -> Result<Option<Comment>> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT * FROM comments WHERE id = ?1",
            [id.to_string()],
            |row| SqliteCommentRepository::row_to_comment(row)
        );
        match result {
            Ok(comment) => Ok(Some(comment)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn get_task_comments(&self, task_id: Uuid) -> Result<Vec<Comment>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT * FROM comments WHERE task_id = ?1 ORDER BY lft"
        )?;
        let comments = stmt.query_map(
            [task_id.to_string()],
            |row| SqliteCommentRepository::row_to_comment(row)
        )?.collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(comments)
    }

    fn get_thread(&self, root_id: Uuid) -> Result<Vec<Comment>> {
        let conn = self.conn.lock().unwrap();
        let root = self.find_by_id(root_id)?;
        match root {
            Some(r) => {
                let mut stmt = conn.prepare(
                    "SELECT * FROM comments WHERE task_id = ?1 AND lft BETWEEN ?2 AND ?3 ORDER BY lft"
                )?;
                let comments = stmt.query_map(
                    [r.task_id.to_string(), r.lft.to_string(), r.rgt.to_string()],
                    |row| SqliteCommentRepository::row_to_comment(row)
                )?.collect::<std::result::Result<Vec<_>, _>>()?;
                Ok(comments)
            }
            None => Ok(vec![]),
        }
    }
}

pub struct SqliteEventLogRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteEventLogRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    fn row_to_event(row: &Row) -> std::result::Result<EventLog, rusqlite::Error> {
        Ok(EventLog {
            id: row.get(0)?,
            user_id: Uuid::parse_str(row.get::<_, String>(1)?.as_str()).unwrap_or_else(|_| Uuid::nil()),
            event_type: row.get(2)?,
            entity_type: row.get(3)?,
            entity_id: row.get(4)?,
            data: serde_json::from_str(row.get::<_, String>(5)?.as_str()).unwrap_or_default(),
            workspace_id: row.get::<_, Option<String>>(6)?.and_then(|s| Uuid::parse_str(&s).ok()),
            created_at: row.get(7)?,
        })
    }
}

impl EventLogRepository for SqliteEventLogRepository {
    fn log(&self, event: EventLog) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO event_log (user_id, event_type, entity_type, entity_id, data, workspace_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                event.user_id.to_string(),
                event.event_type,
                event.entity_type,
                event.entity_id,
                serde_json::to_string(&event.data)?,
                event.workspace_id.map(|id| id.to_string()),
                event.created_at,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    fn get_user_events(&self, user_id: Uuid, limit: i64) -> Result<Vec<EventLog>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT * FROM event_log WHERE user_id = ?1 ORDER BY created_at DESC LIMIT ?2"
        )?;
        let events = stmt.query_map(
            [user_id.to_string(), limit.to_string()],
            |row| SqliteEventLogRepository::row_to_event(row)
        )?.collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(events)
    }

    fn get_workspace_events(&self, workspace_id: Uuid, limit: i64) -> Result<Vec<EventLog>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT * FROM event_log WHERE workspace_id = ?1 ORDER BY created_at DESC LIMIT ?2"
        )?;
        let events = stmt.query_map(
            [workspace_id.to_string(), limit.to_string()],
            |row| SqliteEventLogRepository::row_to_event(row)
        )?.collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(events)
    }

    fn get_entity_events(&self, entity_type: &str, entity_id: &str, limit: i64) -> Result<Vec<EventLog>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT * FROM event_log WHERE entity_type = ?1 AND entity_id = ?2 ORDER BY created_at DESC LIMIT ?3"
        )?;
        let events = stmt.query_map(
            [entity_type, entity_id, limit.to_string().as_str()],
            |row| SqliteEventLogRepository::row_to_event(row)
        )?.collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(events)
    }
}
