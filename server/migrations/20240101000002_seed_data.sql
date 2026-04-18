-- Insert default roles
INSERT INTO roles (id, name, description, is_system) VALUES
    (gen_random_uuid(), 'super_admin', 'Super administrator with all permissions, cannot be deleted or modified', true),
    (gen_random_uuid(), 'admin', 'Full system access including user management', true),
    (gen_random_uuid(), 'editor', 'Can create/edit wiki, import documents, use chat and research', true),
    (gen_random_uuid(), 'viewer', 'Read-only access to wiki, knowledge graph, and search results', true)
ON CONFLICT (name) DO NOTHING;

-- Insert default permissions
INSERT INTO permissions (name, resource, action, description) VALUES
    -- User management
    ('users:read', 'users', 'read', 'View users'),
    ('users:create', 'users', 'create', 'Create users'),
    ('users:update', 'users', 'update', 'Update users'),
    ('users:delete', 'users', 'delete', 'Delete users'),
    
    -- Project management
    ('projects:read', 'projects', 'read', 'View projects'),
    ('projects:create', 'projects', 'create', 'Create projects'),
    ('projects:update', 'projects', 'update', 'Update projects'),
    ('projects:delete', 'projects', 'delete', 'Delete projects'),
    ('projects:manage_members', 'projects', 'manage_members', 'Manage project members'),
    
    -- File operations
    ('files:read', 'files', 'read', 'Read files'),
    ('files:write', 'files', 'write', 'Write files'),
    ('files:delete', 'files', 'delete', 'Delete files'),
    
    -- Wiki operations
    ('wiki:read', 'wiki', 'read', 'Read wiki pages'),
    ('wiki:write', 'wiki', 'write', 'Edit wiki pages'),
    ('wiki:delete', 'wiki', 'delete', 'Delete wiki pages'),
    
    -- Chat
    ('chat:read', 'chat', 'read', 'View chat history'),
    ('chat:write', 'chat', 'write', 'Send chat messages'),
    
    -- Research
    ('research:run', 'research', 'run', 'Run deep research'),
    ('research:read', 'research', 'read', 'View research results'),
    
    -- Ingest
    ('ingest:run', 'ingest', 'run', 'Run document ingest'),
    ('ingest:manage', 'ingest', 'manage', 'Manage ingest queue'),
    
    -- Graph
    ('graph:read', 'graph', 'read', 'View knowledge graph'),
    
    -- Search
    ('search:run', 'search', 'run', 'Run searches'),
    
    -- Review
    ('review:read', 'review', 'read', 'View review items'),
    ('review:manage', 'review', 'manage', 'Manage review items'),
    
    -- Settings
    ('settings:read', 'settings', 'read', 'View settings'),
    ('settings:update', 'settings', 'update', 'Update settings')
ON CONFLICT (name) DO NOTHING;

-- Assign all permissions to super_admin role
INSERT INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'super_admin'),
    id
FROM permissions
ON CONFLICT DO NOTHING;

-- Assign all permissions to admin role
INSERT INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'admin'),
    id
FROM permissions
ON CONFLICT DO NOTHING;

-- Assign editor permissions
INSERT INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'editor'),
    id
FROM permissions
WHERE name NOT IN (
    'users:read', 'users:create', 'users:update', 'users:delete',
    'projects:manage_members'
)
ON CONFLICT DO NOTHING;

-- Assign viewer permissions
INSERT INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'viewer'),
    id
FROM permissions
WHERE name IN (
    'projects:read', 'files:read', 'wiki:read', 'chat:read',
    'research:read', 'graph:read', 'search:run', 'review:read',
    'settings:read'
)
ON CONFLICT DO NOTHING;

-- Create initial super admin user (username: admin, password: admin123)
-- NOTE: Change this password immediately after first login!
INSERT INTO users (id, username, email, password_hash, display_name, is_super_admin, is_active)
VALUES (
    gen_random_uuid(),
    'admin',
    'admin@llmwiki.local',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYILp91S.0i',
    'Super Administrator',
    true,
    true
) ON CONFLICT (username) DO NOTHING;

-- Assign super_admin role to the initial super admin user
INSERT INTO user_roles (user_id, role_id)
SELECT 
    (SELECT id FROM users WHERE username = 'admin' AND is_super_admin = true),
    (SELECT id FROM roles WHERE name = 'super_admin')
ON CONFLICT DO NOTHING;
