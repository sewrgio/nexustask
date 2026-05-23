import { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { Plus, Users, Settings, ChevronRight } from 'lucide-react';
import { workspacesAPI, Workspace } from '../lib/api';

export default function Workspaces() {
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreateForm, setShowCreateForm] = useState(false);

  useEffect(() => {
    loadWorkspaces();
  }, []);

  const loadWorkspaces = async () => {
    try {
      setLoading(true);
      const response = await workspacesAPI.getAll();
      setWorkspaces(response.data);
    } catch (error) {
      console.error('Error loading workspaces:', error);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-slate-500">Cargando espacios de trabajo...</div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-white">Espacios de Trabajo</h2>
          <p className="text-slate-500 text-sm">Gestiona tus espacios de trabajo y equipos</p>
        </div>
        <button
          onClick={() => setShowCreateForm(!showCreateForm)}
          className="bg-blue-600 hover:bg-blue-500 text-white px-4 py-2 rounded-lg text-sm font-bold flex items-center gap-2 transition-all shadow-lg shadow-blue-900/20 active:scale-95"
        >
          <Plus size={18} />
          Nuevo Espacio
        </button>
      </div>

      {showCreateForm && (
        <motion.div
          initial={{ opacity: 0, y: -10 }}
          animate={{ opacity: 1, y: 0 }}
          className="bg-slate-900/50 border border-slate-800 rounded-2xl p-6 backdrop-blur-xl"
        >
          <h3 className="text-lg font-bold text-white mb-4">Crear Nuevo Espacio de Trabajo</h3>
          <CreateWorkspaceForm onSuccess={() => { setShowCreateForm(false); loadWorkspaces(); }} onCancel={() => setShowCreateForm(false)} />
        </motion.div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {workspaces.length === 0 ? (
          <div className="col-span-full text-center py-12">
            <p className="text-slate-500">No hay espacios de trabajo aún. Crea tu primer espacio.</p>
          </div>
        ) : (
          workspaces.map((workspace) => (
            <WorkspaceCard key={workspace.id} workspace={workspace} />
          ))
        )}
      </div>
    </div>
  );
}

function WorkspaceCard({ workspace }: { workspace: Workspace }) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      className="bg-slate-900/50 border border-slate-800 rounded-2xl p-6 backdrop-blur-xl hover:border-slate-700 transition-colors group"
    >
      <div className="flex items-start justify-between mb-4">
        <div className="w-12 h-12 bg-gradient-to-br from-blue-600 to-indigo-600 rounded-xl flex items-center justify-center text-white font-bold text-lg">
          {workspace.name[0]?.toUpperCase() || 'W'}
        </div>
        <button className="p-2 hover:bg-slate-800 rounded-lg transition-colors text-slate-400 hover:text-white">
          <Settings size={16} />
        </button>
      </div>

      <h3 className="text-lg font-bold text-white mb-2">{workspace.name}</h3>
      {workspace.description && (
        <p className="text-sm text-slate-400 mb-4 line-clamp-2">{workspace.description}</p>
      )}

      <div className="flex items-center justify-between pt-4 border-t border-slate-800">
        <div className="flex items-center gap-2 text-slate-500 text-sm">
          <Users size={16} />
          <span>Miembros</span>
        </div>
        <button className="text-blue-400 hover:text-blue-300 text-sm font-medium flex items-center gap-1 transition-colors">
          Ver detalles <ChevronRight size={16} />
        </button>
      </div>
    </motion.div>
  );
}

function CreateWorkspaceForm({ onSuccess, onCancel }: { onSuccess: () => void; onCancel: () => void }) {
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    try {
      await workspacesAPI.create({ name, description });
      onSuccess();
    } catch (error) {
      console.error('Error creating workspace:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div>
        <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2">Nombre</label>
        <input
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          className="w-full bg-slate-800/50 border border-slate-700 rounded-xl px-4 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all"
          placeholder="Nombre del espacio"
          required
        />
      </div>

      <div>
        <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2">Descripción</label>
        <textarea
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          className="w-full bg-slate-800/50 border border-slate-700 rounded-xl px-4 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all"
          placeholder="Descripción del espacio"
          rows={3}
        />
      </div>

      <div className="flex gap-3 pt-4">
        <button
          type="submit"
          disabled={loading}
          className="flex-1 bg-blue-600 hover:bg-blue-500 text-white font-bold py-3 rounded-xl transition-all disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {loading ? 'Creando...' : 'Crear Espacio'}
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
