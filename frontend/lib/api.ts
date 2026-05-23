import axios from 'axios';

const API_BASE_URL = 'http://127.0.0.1:8765/api';

const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Add JWT token to requests
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('jwt_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Handle response errors
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('jwt_token');
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);

export interface User {
  id: string;
  username: string;
  email: string;
  role: string;
  created_at: string;
}

export interface Task {
  id: string;
  title: string;
  description?: string;
  status: string;
  priority: string;
  workspace_id: string;
  parent_id?: string;
  lft: number;
  rgt: number;
  depth: number;
  due_date?: string;
  created_at: string;
  updated_at: string;
  children?: Task[];
}

export interface Workspace {
  id: string;
  name: string;
  slug: string;
  description?: string;
  parent_workspace_id?: string;
  lft: number;
  rgt: number;
  depth: number;
  created_at: string;
}

export interface Comment {
  id: string;
  task_id: string;
  user_id: string;
  content: string;
  parent_id?: string;
  lft: number;
  rgt: number;
  depth: number;
  created_at: string;
  updated_at: string;
}

// Auth API
export const authAPI = {
  register: (username: string, email: string, password: string) =>
    api.post<{ token: string; user: User }>('/users/register', { username, email, password }),
  
  login: (username: string, password: string) =>
    api.post<{ token: string; user: User }>('/users/login', { username, password }),
  
  getCurrentUser: () =>
    api.get<User>('/users/me'),
};

// Tasks API
export const tasksAPI = {
  getAll: () => api.get<Task[]>('/tasks'),
  
  getById: (id: string) => api.get<Task>(`/tasks/${id}`),
  
  getTree: (id: string) => api.get<Task[]>(`/tasks/${id}/tree`),
  
  getByWorkspace: (workspaceId: string) => api.get<Task[]>(`/workspaces/${workspaceId}/tasks`),
  
  create: (data: { title: string; description?: string; status: string; priority: string; workspace_id: string; parent_id?: string; due_date?: string }) =>
    api.post<Task>('/tasks', data),
  
  update: (id: string, data: Partial<Task>) => api.put<Task>(`/tasks/${id}`, data),
  
  delete: (id: string) => api.delete(`/tasks/${id}`),
  
  move: (id: string, data: { new_parent_id?: string; new_position?: number }) =>
    api.patch<Task>(`/tasks/${id}/move`, data),
  
  updateStatus: (id: string, status: string) =>
    api.patch<Task>(`/tasks/${id}/status`, { status }),
  
  getComments: (taskId: string) => api.get<Comment[]>(`/tasks/${taskId}/comments`),
  
  addComment: (taskId: string, content: string) =>
    api.post<Comment>(`/tasks/${taskId}/comments`, { content }),
};

// Workspaces API
export const workspacesAPI = {
  getAll: () => api.get<Workspace[]>('/workspaces'),
  
  getById: (id: string) => api.get<Workspace>(`/workspaces/${id}`),
  
  create: (data: { name: string; description?: string; parent_workspace_id?: string }) =>
    api.post<Workspace>('/workspaces', data),
  
  getMembers: (id: string) => api.get(`/workspaces/${id}/members`),
  
  addMember: (id: string, userId: string, role: string) =>
    api.post(`/workspaces/${id}/members`, { user_id: userId, role }),
  
  removeMember: (workspaceId: string, userId: string) =>
    api.delete(`/workspaces/${workspaceId}/members/${userId}`),
  
  updateMemberRole: (workspaceId: string, userId: string, role: string) =>
    api.patch(`/workspaces/${workspaceId}/members/${userId}/role`, { role }),
};

export default api;
