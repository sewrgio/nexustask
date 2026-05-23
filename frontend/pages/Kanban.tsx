import { useState, useEffect } from 'react';
import { Plus } from 'lucide-react';
import { tasksAPI, Task } from '../lib/api';

export default function Kanban() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [loading, setLoading] = useState(true);

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

  const columns = [
    { id: 'pending', title: 'Pendiente', tasks: tasks.filter(t => t.status === 'pending') },
    { id: 'in_progress', title: 'En Progreso', tasks: tasks.filter(t => t.status === 'in_progress') },
    { id: 'completed', title: 'Completado', tasks: tasks.filter(t => t.status === 'completed') },
    { id: 'cancelled', title: 'Cancelado', tasks: tasks.filter(t => t.status === 'cancelled') },
  ];

  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case 'critical': return 'text-red-400';
      case 'high': return 'text-orange-400';
      case 'medium': return 'text-yellow-400';
      case 'low': return 'text-green-400';
      default: return 'text-slate-400';
    }
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
          <h2 className="text-2xl font-bold text-white">Tablero Kanban</h2>
          <p className="text-slate-500 text-sm">Visualiza y gestiona el flujo de trabajo</p>
        </div>
        <button className="bg-blue-600 hover:bg-blue-500 text-white px-4 py-2 rounded-lg text-sm font-bold flex items-center gap-2 transition-all shadow-lg shadow-blue-900/20 active:scale-95">
          <Plus size={18} />
          Nueva Tarea
        </button>
      </div>

      <div className="flex gap-4 overflow-x-auto pb-4">
        {columns.map((column) => (
          <div
            key={column.id}
            className="flex-shrink-0 w-80 bg-slate-900/50 border border-slate-800 rounded-2xl p-4 backdrop-blur-xl"
          >
            <div className="flex items-center justify-between mb-4">
              <h3 className="font-bold text-white">{column.title}</h3>
              <span className="text-xs text-slate-500 bg-slate-800 px-2 py-1 rounded-full">
                {column.tasks.length}
              </span>
            </div>
            
            <div className="space-y-3 min-h-[200px]">
              {column.tasks.length === 0 ? (
                <div className="text-center py-8 text-slate-600 text-sm">
                  Sin tareas
                </div>
              ) : (
                column.tasks.map((task) => (
                  <div
                    key={task.id}
                    className="bg-slate-800/50 border border-slate-700 rounded-xl p-4 hover:border-slate-600 transition-colors cursor-move"
                  >
                    <p className="text-sm font-medium text-white mb-2">{task.title}</p>
                    {task.description && (
                      <p className="text-xs text-slate-400 mb-3 line-clamp-2">{task.description}</p>
                    )}
                    <div className="flex items-center justify-between">
                      <span className={`text-xs font-medium ${getPriorityColor(task.priority)}`}>
                        {task.priority}
                      </span>
                      {task.due_date && (
                        <span className="text-xs text-slate-500">
                          {new Date(task.due_date).toLocaleDateString()}
                        </span>
                      )}
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
