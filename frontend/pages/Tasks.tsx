import { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { Plus, Edit2, Trash2, ChevronRight, ChevronDown, MessageSquare } from 'lucide-react';
import { tasksAPI, Task, Comment } from '../lib/api';

export default function Tasks() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [showEditForm, setShowEditForm] = useState(false);
  const [editingTask, setEditingTask] = useState<Task | null>(null);
  const [expandedTasks, setExpandedTasks] = useState<Set<string>>(new Set());
  const [selectedTaskComments, setSelectedTaskComments] = useState<Comment[]>([]);
  const [showComments, setShowComments] = useState(false);
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null);

  useEffect(() => {
    loadTasks();
  }, []);

  const loadTasks = async () => {
    try {
      setLoading(true);
      const response = await tasksAPI.getAll();
      setTasks(response.data);
    } catch (error) {
      console.error('Error loading tasks:', error);
    } finally {
      setLoading(false);
    }
  };

  const toggleExpand = (taskId: string) => {
    const newExpanded = new Set(expandedTasks);
    if (newExpanded.has(taskId)) {
      newExpanded.delete(taskId);
    } else {
      newExpanded.add(taskId);
    }
    setExpandedTasks(newExpanded);
  };

  const handleEditTask = (task: Task) => {
    setEditingTask(task);
    setShowEditForm(true);
  };

  const handleDeleteTask = async (taskId: string) => {
    if (!confirm('¿Está seguro de eliminar esta tarea?')) return;
    
    try {
      await tasksAPI.delete(taskId);
      loadTasks();
    } catch (error) {
      console.error('Error deleting task:', error);
    }
  };

  const handleShowComments = async (taskId: string) => {
    setSelectedTaskId(taskId);
    setShowComments(true);
    try {
      const response = await tasksAPI.getComments(taskId);
      setSelectedTaskComments(response.data);
    } catch (error) {
      console.error('Error loading comments:', error);
    }
  };

  const handleAddComment = async (content: string) => {
    if (!selectedTaskId) return;
    
    try {
      await tasksAPI.addComment(selectedTaskId, content);
      const response = await tasksAPI.getComments(selectedTaskId);
      setSelectedTaskComments(response.data);
    } catch (error) {
      console.error('Error adding comment:', error);
    }
  };

  const renderTaskTree = (task: Task, level: number = 0) => {
    const hasChildren = task.children && task.children.length > 0;
    const isExpanded = expandedTasks.has(task.id);

    return (
      <div key={task.id} className="ml-4">
        <div
          className="flex items-center gap-2 p-3 bg-slate-900/30 border border-slate-800 rounded-lg hover:border-slate-700 transition-colors"
          style={{ marginLeft: `${level * 16}px` }}
        >
          {hasChildren && (
            <button
              onClick={() => toggleExpand(task.id)}
              className="p-1 hover:bg-slate-800 rounded transition-colors"
            >
              {isExpanded ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
            </button>
          )}
          {!hasChildren && <div className="w-6" />}
          
          <div className="flex-1">
            <p className="text-sm font-medium text-white">{task.title}</p>
            <p className="text-xs text-slate-500">{task.status} • {task.priority}</p>
          </div>
          
          <div className="flex items-center gap-2">
            <button onClick={() => handleShowComments(task.id)} className="p-2 hover:bg-slate-800 rounded-lg transition-colors text-slate-400 hover:text-white">
              <MessageSquare size={16} />
            </button>
            <button onClick={() => handleEditTask(task)} className="p-2 hover:bg-slate-800 rounded-lg transition-colors text-slate-400 hover:text-white">
              <Edit2 size={16} />
            </button>
            <button onClick={() => handleDeleteTask(task.id)} className="p-2 hover:bg-slate-800 rounded-lg transition-colors text-slate-400 hover:text-red-400">
              <Trash2 size={16} />
            </button>
          </div>
        </div>
        
        {hasChildren && isExpanded && (
          <div className="mt-2">
            {task.children?.map(child => renderTaskTree(child, level + 1))}
          </div>
        )}
      </div>
    );
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-slate-500">Cargando tareas...</div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-white">Árbol de Tareas</h2>
          <p className="text-slate-500 text-sm">Gestiona tu jerarquía de tareas</p>
        </div>
        <button
          onClick={() => setShowCreateForm(!showCreateForm)}
          className="bg-blue-600 hover:bg-blue-500 text-white px-4 py-2 rounded-lg text-sm font-bold flex items-center gap-2 transition-all shadow-lg shadow-blue-900/20 active:scale-95"
        >
          <Plus size={18} />
          Nueva Tarea
        </button>
      </div>

      {showCreateForm && (
        <motion.div
          initial={{ opacity: 0, y: -10 }}
          animate={{ opacity: 1, y: 0 }}
          className="bg-slate-900/50 border border-slate-800 rounded-2xl p-6 backdrop-blur-xl"
        >
          <h3 className="text-lg font-bold text-white mb-4">Crear Nueva Tarea</h3>
          <CreateTaskForm onSuccess={() => { setShowCreateForm(false); loadTasks(); }} />
        </motion.div>
      )}

      {showEditForm && editingTask && (
        <motion.div
          initial={{ opacity: 0, y: -10 }}
          animate={{ opacity: 1, y: 0 }}
          className="bg-slate-900/50 border border-slate-800 rounded-2xl p-6 backdrop-blur-xl"
        >
          <h3 className="text-lg font-bold text-white mb-4">Editar Tarea</h3>
          <EditTaskForm task={editingTask} onSuccess={() => { setShowEditForm(false); setEditingTask(null); loadTasks(); }} onCancel={() => { setShowEditForm(false); setEditingTask(null); }} />
        </motion.div>
      )}

      {showComments && (
        <motion.div
          initial={{ opacity: 0, y: -10 }}
          animate={{ opacity: 1, y: 0 }}
          className="bg-slate-900/50 border border-slate-800 rounded-2xl p-6 backdrop-blur-xl"
        >
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-bold text-white">Comentarios</h3>
            <button onClick={() => setShowComments(false)} className="text-slate-400 hover:text-white transition-colors">
              ✕
            </button>
          </div>
          <CommentsPanel comments={selectedTaskComments} onAddComment={handleAddComment} />
        </motion.div>
      )}

      <div className="bg-slate-900/50 border border-slate-800 rounded-2xl p-6 backdrop-blur-xl">
        {tasks.length === 0 ? (
          <div className="text-center py-12">
            <p className="text-slate-500">No hay tareas aún. Crea tu primera tarea.</p>
          </div>
        ) : (
          <div className="space-y-2">
            {tasks.map(task => renderTaskTree(task))}
          </div>
        )}
      </div>
    </div>
  );
}

function CreateTaskForm({ onSuccess }: { onSuccess: () => void }) {
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [status, setStatus] = useState('pending');
  const [priority, setPriority] = useState('medium');
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    try {
      await tasksAPI.create({
        title,
        description,
        status,
        priority,
        workspace_id: 'default', // TODO: Get from context
      });
      onSuccess();
    } catch (error) {
      console.error('Error creating task:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div>
        <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2">Título</label>
        <input
          type="text"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          className="w-full bg-slate-800/50 border border-slate-700 rounded-xl px-4 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all"
          placeholder="Título de la tarea"
          required
        />
      </div>
      
      <div>
        <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2">Descripción</label>
        <textarea
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          className="w-full bg-slate-800/50 border border-slate-700 rounded-xl px-4 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all"
          placeholder="Descripción de la tarea"
          rows={3}
        />
      </div>

      <div className="grid grid-cols-2 gap-4">
        <div>
          <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2">Estado</label>
          <select
            value={status}
            onChange={(e) => setStatus(e.target.value)}
            className="w-full bg-slate-800/50 border border-slate-700 rounded-xl px-4 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all"
          >
            <option value="pending">Pendiente</option>
            <option value="in_progress">En Progreso</option>
            <option value="completed">Completado</option>
            <option value="cancelled">Cancelado</option>
          </select>
        </div>
        
        <div>
          <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2">Prioridad</label>
          <select
            value={priority}
            onChange={(e) => setPriority(e.target.value)}
            className="w-full bg-slate-800/50 border border-slate-700 rounded-xl px-4 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all"
          >
            <option value="low">Baja</option>
            <option value="medium">Media</option>
            <option value="high">Alta</option>
            <option value="critical">Crítica</option>
          </select>
        </div>
      </div>

      <div className="flex gap-3 pt-4">
        <button
          type="submit"
          disabled={loading}
          className="flex-1 bg-blue-600 hover:bg-blue-500 text-white font-bold py-3 rounded-xl transition-all disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {loading ? 'Creando...' : 'Crear Tarea'}
        </button>
        <button
          type="button"
          onClick={onSuccess}
          className="px-6 bg-slate-800 hover:bg-slate-700 text-white font-bold py-3 rounded-xl transition-all"
        >
          Cancelar
        </button>
      </div>
    </form>
  );
}

function EditTaskForm({ task, onSuccess, onCancel }: { task: Task; onSuccess: () => void; onCancel: () => void }) {
  const [title, setTitle] = useState(task.title);
  const [description, setDescription] = useState(task.description || '');
  const [status, setStatus] = useState(task.status);
  const [priority, setPriority] = useState(task.priority);
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    try {
      await tasksAPI.update(task.id, {
        title,
        description,
        status,
        priority,
      });
      onSuccess();
    } catch (error) {
      console.error('Error updating task:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div>
        <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2">Título</label>
        <input
          type="text"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          className="w-full bg-slate-800/50 border border-slate-700 rounded-xl px-4 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all"
          placeholder="Título de la tarea"
          required
        />
      </div>
      
      <div>
        <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2">Descripción</label>
        <textarea
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          className="w-full bg-slate-800/50 border border-slate-700 rounded-xl px-4 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all"
          placeholder="Descripción de la tarea"
          rows={3}
        />
      </div>

      <div className="grid grid-cols-2 gap-4">
        <div>
          <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2">Estado</label>
          <select
            value={status}
            onChange={(e) => setStatus(e.target.value)}
            className="w-full bg-slate-800/50 border border-slate-700 rounded-xl px-4 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all"
          >
            <option value="pending">Pendiente</option>
            <option value="in_progress">En Progreso</option>
            <option value="completed">Completado</option>
            <option value="cancelled">Cancelado</option>
          </select>
        </div>
        
        <div>
          <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2">Prioridad</label>
          <select
            value={priority}
            onChange={(e) => setPriority(e.target.value)}
            className="w-full bg-slate-800/50 border border-slate-700 rounded-xl px-4 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all"
          >
            <option value="low">Baja</option>
            <option value="medium">Media</option>
            <option value="high">Alta</option>
            <option value="critical">Crítica</option>
          </select>
        </div>
      </div>

      <div className="flex gap-3 pt-4">
        <button
          type="submit"
          disabled={loading}
          className="flex-1 bg-blue-600 hover:bg-blue-500 text-white font-bold py-3 rounded-xl transition-all disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {loading ? 'Guardando...' : 'Guardar Cambios'}
        </button>
        <button
          type="button"
          onClick={onCancel}
          className="px-6 bg-slate-800 hover:bg-slate-700 text-white font-bold py-3 rounded-xl transition-all"
        >
          Cancelar
        </button>
      </div>
    </form>
  );
}

function CommentsPanel({ comments, onAddComment }: { comments: Comment[]; onAddComment: (content: string) => void }) {
  const [newComment, setNewComment] = useState('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (newComment.trim()) {
      onAddComment(newComment);
      setNewComment('');
    }
  };

  return (
    <div className="space-y-4">
      <div className="space-y-3 max-h-96 overflow-y-auto">
        {comments.length === 0 ? (
          <p className="text-slate-500 text-sm text-center py-4">No hay comentarios aún</p>
        ) : (
          comments.map((comment) => (
            <div key={comment.id} className="bg-slate-800/50 border border-slate-700 rounded-xl p-4">
              <div className="flex items-start gap-3">
                <div className="w-8 h-8 rounded-full bg-blue-600 flex items-center justify-center text-white text-xs font-bold">
                  {comment.user_id[0]?.toUpperCase() || 'U'}
                </div>
                <div className="flex-1">
                  <p className="text-sm text-white">{comment.content}</p>
                  <p className="text-xs text-slate-500 mt-1">
                    {new Date(comment.created_at).toLocaleString()}
                  </p>
                </div>
              </div>
            </div>
          ))
        )}
      </div>

      <form onSubmit={handleSubmit} className="flex gap-3">
        <input
          type="text"
          value={newComment}
          onChange={(e) => setNewComment(e.target.value)}
          placeholder="Escribe un comentario..."
          className="flex-1 bg-slate-800/50 border border-slate-700 rounded-xl px-4 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all"
        />
        <button
          type="submit"
          className="bg-blue-600 hover:bg-blue-500 text-white px-4 py-2 rounded-xl font-bold transition-all"
        >
          Enviar
        </button>
      </form>
    </div>
  );
}
