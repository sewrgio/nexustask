import { useState } from 'react';
import { motion } from 'framer-motion';
import { Users, Database, Shield, Settings as SettingsIcon, Download, RefreshCw, Trash2 } from 'lucide-react';

export default function Admin() {
  const [activeTab, setActiveTab] = useState('users');

  const tabs = [
    { id: 'users', label: 'Usuarios', icon: <Users size={18} /> },
    { id: 'backups', label: 'Backups', icon: <Database size={18} /> },
    { id: 'security', label: 'Seguridad', icon: <Shield size={18} /> },
    { id: 'settings', label: 'Configuración', icon: <SettingsIcon size={18} /> },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-white">Panel de Administración</h2>
        <p className="text-slate-500 text-sm">Gestión del sistema y configuraciones globales</p>
      </div>

      <div className="bg-slate-900/50 border border-slate-800 rounded-2xl backdrop-blur-xl">
        <div className="flex border-b border-slate-800">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`flex items-center gap-2 px-6 py-4 text-sm font-medium transition-colors ${
                activeTab === tab.id
                  ? 'text-blue-400 border-b-2 border-blue-400 bg-slate-800/50'
                  : 'text-slate-400 hover:text-white hover:bg-slate-800/30'
              }`}
            >
              {tab.icon}
              {tab.label}
            </button>
          ))}
        </div>

        <div className="p-6">
          {activeTab === 'users' && <UsersManagement />}
          {activeTab === 'backups' && <BackupsManagement />}
          {activeTab === 'security' && <SecuritySettings />}
          {activeTab === 'settings' && <SystemSettings />}
        </div>
      </div>
    </div>
  );
}

function UsersManagement() {
  const [users] = useState([
    { id: '1', username: 'admin', email: 'admin@nexustask.com', role: 'super_admin', created_at: '2024-01-15' },
    { id: '2', username: 'sergio', email: 'sergio@nexustask.com', role: 'admin', created_at: '2024-02-20' },
    { id: '3', username: 'maria', email: 'maria@nexustask.com', role: 'manager', created_at: '2024-03-10' },
  ]);

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between mb-6">
        <h3 className="text-lg font-bold text-white">Gestión de Usuarios</h3>
        <button className="bg-blue-600 hover:bg-blue-500 text-white px-4 py-2 rounded-lg text-sm font-bold flex items-center gap-2 transition-all">
          <Users size={16} />
          Nuevo Usuario
        </button>
      </div>

      <div className="space-y-3">
        {users.map((user) => (
          <div key={user.id} className="bg-slate-800/50 border border-slate-700 rounded-xl p-4 flex items-center justify-between">
            <div className="flex items-center gap-4">
              <div className="w-10 h-10 rounded-full bg-gradient-to-tr from-blue-600 to-indigo-600 flex items-center justify-center text-white font-bold">
                {user.username[0]?.toUpperCase()}
              </div>
              <div>
                <p className="text-sm font-medium text-white">{user.username}</p>
                <p className="text-xs text-slate-500">{user.email}</p>
              </div>
            </div>
            <div className="flex items-center gap-4">
              <span className="text-xs px-2 py-1 rounded-full bg-blue-500/20 text-blue-400 font-medium">
                {user.role}
              </span>
              <button className="p-2 hover:bg-slate-700 rounded-lg transition-colors text-slate-400 hover:text-white">
                <SettingsIcon size={16} />
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

function BackupsManagement() {
  const [backups] = useState([
    { id: '1', filename: 'backup_2024-05-23_02-00.sql', size: '2.4 MB', created_at: '2024-05-23 02:00' },
    { id: '2', filename: 'backup_2024-05-22_02-00.sql', size: '2.3 MB', created_at: '2024-05-22 02:00' },
    { id: '3', filename: 'backup_2024-05-21_02-00.sql', size: '2.2 MB', created_at: '2024-05-21 02:00' },
  ]);

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between mb-6">
        <h3 className="text-lg font-bold text-white">Gestión de Backups</h3>
        <button className="bg-blue-600 hover:bg-blue-500 text-white px-4 py-2 rounded-lg text-sm font-bold flex items-center gap-2 transition-all">
          <RefreshCw size={16} />
          Crear Backup
        </button>
      </div>

      <div className="space-y-3">
        {backups.map((backup) => (
          <div key={backup.id} className="bg-slate-800/50 border border-slate-700 rounded-xl p-4 flex items-center justify-between">
            <div className="flex items-center gap-4">
              <div className="w-10 h-10 rounded-lg bg-emerald-500/20 flex items-center justify-center text-emerald-400">
                <Database size={20} />
              </div>
              <div>
                <p className="text-sm font-medium text-white">{backup.filename}</p>
                <p className="text-xs text-slate-500">{backup.size} • {backup.created_at}</p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <button className="p-2 hover:bg-slate-700 rounded-lg transition-colors text-slate-400 hover:text-white">
                <Download size={16} />
              </button>
              <button className="p-2 hover:bg-slate-700 rounded-lg transition-colors text-slate-400 hover:text-red-400">
                <Trash2 size={16} />
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

function SecuritySettings() {
  return (
    <div className="space-y-6">
      <h3 className="text-lg font-bold text-white">Configuración de Seguridad</h3>

      <div className="space-y-4">
        <div className="bg-slate-800/50 border border-slate-700 rounded-xl p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-white">Autenticación de dos factores</p>
              <p className="text-xs text-slate-500">Requerir 2FA para todos los usuarios</p>
            </div>
            <button className="w-12 h-6 bg-slate-700 rounded-full relative transition-colors">
              <div className="w-5 h-5 bg-white rounded-full absolute top-0.5 left-0.5 transition-transform" />
            </button>
          </div>
        </div>

        <div className="bg-slate-800/50 border border-slate-700 rounded-xl p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-white">Sesiones concurrentes</p>
              <p className="text-xs text-slate-500">Limitar sesiones simultáneas por usuario</p>
            </div>
            <button className="w-12 h-6 bg-blue-600 rounded-full relative transition-colors">
              <div className="w-5 h-5 bg-white rounded-full absolute top-0.5 right-0.5 transition-transform" />
            </button>
          </div>
        </div>

        <div className="bg-slate-800/50 border border-slate-700 rounded-xl p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-white">Log de auditoría</p>
              <p className="text-xs text-slate-500">Registrar todas las acciones administrativas</p>
            </div>
            <button className="w-12 h-6 bg-blue-600 rounded-full relative transition-colors">
              <div className="w-5 h-5 bg-white rounded-full absolute top-0.5 right-0.5 transition-transform" />
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

function SystemSettings() {
  return (
    <div className="space-y-6">
      <h3 className="text-lg font-bold text-white">Configuración del Sistema</h3>

      <div className="space-y-4">
        <div className="bg-slate-800/50 border border-slate-700 rounded-xl p-4">
          <label className="block text-sm font-medium text-white mb-2">Nombre del Sistema</label>
          <input
            type="text"
            defaultValue="NexusTask Enterprise"
            className="w-full bg-slate-900/50 border border-slate-600 rounded-lg px-4 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all"
          />
        </div>

        <div className="bg-slate-800/50 border border-slate-700 rounded-xl p-4">
          <label className="block text-sm font-medium text-white mb-2">Límite de Tareas por Usuario</label>
          <input
            type="number"
            defaultValue="1000"
            className="w-full bg-slate-900/50 border border-slate-600 rounded-lg px-4 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all"
          />
        </div>

        <div className="bg-slate-800/50 border border-slate-700 rounded-xl p-4">
          <label className="block text-sm font-medium text-white mb-2">Retención de Backups (días)</label>
          <input
            type="number"
            defaultValue="30"
            className="w-full bg-slate-900/50 border border-slate-600 rounded-lg px-4 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all"
          />
        </div>

        <button className="bg-blue-600 hover:bg-blue-500 text-white px-6 py-2 rounded-lg font-bold transition-all">
          Guardar Cambios
        </button>
      </div>
    </div>
  );
}
