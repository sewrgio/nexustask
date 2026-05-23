import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { LayoutDashboard, CheckSquare, Kanban, BarChart3, Settings, ShieldAlert, LogOut, Search, Plus, Bell, User } from "lucide-react";

// Mock data for the Luxury Demo
const stats = [
  { label: "TOTAL TAREAS", value: "1,284", trend: "+12% desde el mes pasado", color: "text-blue-400" },
  { label: "TASA COMPLETADO", value: "94.2%", trend: "+2.4% de mejora", color: "text-emerald-400" },
  { label: "RUTA CRÍTICA", value: "14", trend: "7 requieren atención", color: "text-amber-400" },
];

export default function App() {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [activeModule, setActiveModule] = useState("dashboard");

  if (!isAuthenticated) {
    return <Login onLogin={() => setIsAuthenticated(true)} />;
  }

  return (
    <div className="flex h-screen w-full bg-slate-950 text-slate-200 overflow-hidden">
      {/* Sidebar */}
      <aside className="w-64 bg-slate-900/50 border-r border-slate-800 flex flex-col backdrop-blur-xl">
        <div className="p-6 flex items-center gap-3">
          <div className="w-8 h-8 bg-blue-600 rounded-lg flex items-center justify-center font-bold text-white">N</div>
          <h1 className="text-xl font-bold tracking-tight text-white">NEXUSTASK</h1>
        </div>

        <nav className="flex-1 px-4 py-4 space-y-1">
          <p className="text-[10px] font-bold text-slate-500 uppercase tracking-widest px-2 mb-4">Menú Principal</p>
          <NavItem icon={<LayoutDashboard size={18}/>} label="Panel Control" active={activeModule === "dashboard"} onClick={() => setActiveModule("dashboard")} />
          <NavItem icon={<CheckSquare size={18}/>} label="Árbol Tareas" active={activeModule === "tasks"} onClick={() => setActiveModule("tasks")} />
          <NavItem icon={<Kanban size={18}/>} label="Tablero Kanban" active={activeModule === "kanban"} onClick={() => setActiveModule("kanban")} />
          <NavItem icon={<BarChart3 size={18}/>} label="Analíticas" active={activeModule === "reports"} onClick={() => setActiveModule("reports")} />
          
          <div className="pt-8 pb-4">
             <p className="text-[10px] font-bold text-slate-500 uppercase tracking-widest px-2 mb-4">Sistema</p>
             <NavItem icon={<ShieldAlert size={18}/>} label="Panel Admin" active={activeModule === "admin"} onClick={() => setActiveModule("admin")} />
             <NavItem icon={<Settings size={18}/>} label="Ajustes" active={activeModule === "settings"} onClick={() => setActiveModule("settings")} />
          </div>
        </nav>

        <div className="p-4 border-t border-slate-800 bg-slate-900/80">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-full bg-gradient-to-tr from-blue-600 to-indigo-600 flex items-center justify-center font-bold text-white shadow-lg">S</div>
            <div className="flex-1 min-w-0">
              <p className="text-sm font-bold text-white truncate">Sergio</p>
              <p className="text-[10px] text-slate-500">Super Administrador</p>
            </div>
            <button className="p-2 hover:bg-slate-800 rounded-lg transition-colors text-slate-400">
              <LogOut size={16} />
            </button>
          </div>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 flex flex-col min-w-0 bg-[radial-gradient(ellipse_at_top_right,_var(--tw-gradient-stops))] from-slate-900 via-slate-950 to-slate-950">
        <header className="h-16 border-b border-slate-800 flex items-center justify-between px-8 backdrop-blur-md bg-slate-950/50 sticky top-0 z-10">
          <div className="flex items-center gap-4 flex-1 max-w-xl">
             <div className="relative w-full">
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" size={16} />
                <input 
                  type="text" 
                  placeholder="Buscar tareas, inteligencia, archivos..." 
                  className="w-full bg-slate-900/50 border border-slate-800 rounded-full py-1.5 pl-10 pr-4 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all placeholder:text-slate-600"
                />
             </div>
          </div>
          
          <div className="flex items-center gap-3">
            <button className="p-2 text-slate-400 hover:text-white transition-colors relative">
               <Bell size={20} />
               <span className="absolute top-2 right-2 w-2 h-2 bg-red-500 rounded-full border-2 border-slate-950"></span>
            </button>
            <button className="bg-blue-600 hover:bg-blue-500 text-white px-4 py-2 rounded-lg text-sm font-bold flex items-center gap-2 transition-all shadow-lg shadow-blue-900/20 active:scale-95">
              <Plus size={18} />
              Nueva Tarea
            </button>
          </div>
        </header>

        <div className="flex-1 overflow-y-auto p-8">
           <AnimatePresence mode="wait">
             {activeModule === "dashboard" && (
               <motion.div 
                 initial={{ opacity: 0, y: 20 }}
                 animate={{ opacity: 1, y: 0 }}
                 exit={{ opacity: 0, y: -20 }}
                 className="space-y-8"
               >
                 {/* Welcome Banner */}
                 <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-blue-600 to-indigo-700 p-8 shadow-2xl shadow-blue-900/20">
                    <div className="relative z-10">
                      <h2 className="text-3xl font-bold text-white mb-2">¡Bienvenido de nuevo, Sergio!</h2>
                      <p className="text-blue-100 opacity-90 max-w-md text-sm">Tu ecosistema empresarial está operando al 98% de eficiencia. Tienes 14 tareas prioritarias que requieren validación.</p>
                      <button className="mt-6 bg-white/10 hover:bg-white/20 text-white px-5 py-2 rounded-xl text-sm font-bold backdrop-blur-md transition-all border border-white/20 active:scale-95">
                        Ver Reporte Detallado
                      </button>
                    </div>
                    {/* Decorative elements */}
                    <div className="absolute top-[-20%] right-[-10%] w-64 h-64 bg-white/10 rounded-full blur-3xl"></div>
                    <div className="absolute bottom-[-40%] left-[20%] w-96 h-96 bg-indigo-500/20 rounded-full blur-3xl"></div>
                 </div>

                 <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                    {stats.map((stat, i) => (
                      <StatCard key={i} {...stat} delay={i * 0.1} />
                    ))}
                 </div>

                 <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
                    <div className="bg-slate-900/50 border border-slate-800 rounded-3xl p-6 backdrop-blur-xl">
                       <h3 className="text-lg font-bold text-white mb-6">Distribución de Carga Semanal</h3>
                       <div className="h-64 flex items-end gap-3 px-4">
                          {[60, 85, 70, 95, 75, 40, 20].map((h, i) => (
                            <motion.div 
                              key={i}
                              initial={{ height: 0 }}
                              animate={{ height: `${h}%` }}
                              transition={{ delay: i * 0.1, duration: 1, ease: "easeOut" }}
                              className={`flex-1 rounded-t-lg bg-gradient-to-t ${i > 4 ? 'from-emerald-600 to-emerald-400' : 'from-blue-600 to-blue-400'} opacity-80 hover:opacity-100 transition-opacity relative group`}
                            >
                              <div className="absolute -top-10 left-1/2 -translate-x-1/2 bg-slate-800 text-white text-[10px] px-2 py-1 rounded opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none">
                                {h}%
                              </div>
                            </motion.div>
                          ))}
                       </div>
                       <div className="flex justify-between mt-4 px-4 text-[10px] text-slate-500 font-bold">
                          {["LUN", "MAR", "MIÉ", "JUE", "VIE", "SÁB", "DOM"].map(d => <span key={d}>{d}</span>)}
                       </div>
                    </div>

                    <div className="bg-slate-900/50 border border-slate-800 rounded-3xl p-6 backdrop-blur-xl">
                       <h3 className="text-lg font-bold text-white mb-6">Actividad Empresarial Reciente</h3>
                       <div className="space-y-6">
                          <ActivityItem icon="🟢" text="Respaldo del sistema completado con éxito" time="hace 2 min" />
                          <ActivityItem icon="🔵" text="Nuevo espacio 'Marketing' creado por Admin" time="hace 15 min" />
                          <ActivityItem icon="🔴" text="Tarea crítica 'Migración Servidor' VENCIDA" time="hace 1 hora" />
                          <ActivityItem icon="🟡" text="Usuario 'Maria' invitado a 'Ingeniería'" time="hace 3 horas" />
                       </div>
                    </div>
                 </div>
               </motion.div>
             )}
           </AnimatePresence>
        </div>
      </main>
    </div>
  );
}

function Login({ onLogin }: { onLogin: () => void }) {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");

  return (
    <div className="h-screen w-full bg-slate-950 flex items-center justify-center p-6 relative overflow-hidden">
      {/* Background Decor */}
      <div className="absolute top-0 left-0 w-full h-full opacity-30 pointer-events-none">
         <div className="absolute top-[-10%] right-[-10%] w-[500px] h-[500px] bg-blue-600/20 rounded-full blur-[120px]"></div>
         <div className="absolute bottom-[-10%] left-[-10%] w-[500px] h-[500px] bg-indigo-600/20 rounded-full blur-[120px]"></div>
      </div>

      <motion.div 
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        className="w-full max-w-md z-10"
      >
        <div className="flex flex-col items-center mb-12">
          <div className="w-16 h-16 bg-blue-600 rounded-2xl flex items-center justify-center font-black text-3xl text-white shadow-2xl shadow-blue-500/50 mb-6">N</div>
          <h1 className="text-4xl font-black tracking-tight text-white mb-2">NEXUS<span className="text-blue-500">TASK</span></h1>
          <p className="text-slate-500 font-medium">Inteligencia de Proyectos Empresariales</p>
        </div>

        <div className="bg-slate-900/80 border border-slate-800 p-10 rounded-[40px] shadow-2xl backdrop-blur-2xl">
          <h2 className="text-2xl font-bold text-white mb-2">Autenticación Requerida</h2>
          <p className="text-slate-500 text-sm mb-8">Inicie sesión para acceder a su ecosistema.</p>
          
          <div className="space-y-6">
            <div>
              <label className="block text-[11px] font-bold text-slate-500 uppercase tracking-widest mb-2 ml-1">Usuario</label>
              <input 
                type="text" 
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                className="w-full bg-slate-800/50 border border-slate-700 rounded-2xl px-5 py-3 text-white focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all placeholder:text-slate-600"
                placeholder="usuario_id"
              />
            </div>
            
            <div>
              <label className="block text-[11px] font-bold text-slate-500 uppercase tracking-widest mb-2 ml-1">Contraseña</label>
              <input 
                type="password" 
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="w-full bg-slate-800/50 border border-slate-700 rounded-2xl px-5 py-3 text-white focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all placeholder:text-slate-600"
                placeholder="••••••••••••"
              />
            </div>

            <button 
              onClick={onLogin}
              className="w-full bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-50 hover:to-indigo-500 text-white font-bold py-4 rounded-2xl shadow-xl shadow-blue-900/40 transition-all active:scale-95 flex items-center justify-center gap-2 mt-4"
            >
              Iniciar Sesión Segura
            </button>

            <div className="pt-6 text-center">
               <a href="#" className="text-xs text-slate-500 hover:text-blue-400 transition-colors">¿Problemas para acceder? Contacte a IT</a>
            </div>
          </div>
        </div>
        
        <p className="mt-12 text-center text-[10px] text-slate-600 uppercase tracking-[0.2em] font-bold">
          v2.0.0-enterprise | Protocolo Seguro v4.2
        </p>
      </motion.div>
    </div>
  );
}

function NavItem({ icon, label, active, onClick }: { icon: any, label: string, active: boolean, onClick: () => void }) {
  return (
    <button 
      onClick={onClick}
      className={`w-full flex items-center gap-3 px-4 py-2.5 rounded-xl text-sm font-medium transition-all ${active ? 'bg-blue-600 text-white shadow-lg shadow-blue-900/20' : 'text-slate-400 hover:text-white hover:bg-slate-800/50'}`}
    >
      {icon}
      {label}
    </button>
  );
}

function StatCard({ label, value, trend, color, delay }: any) {
  return (
    <motion.div 
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay }}
      className="bg-slate-900/50 border border-slate-800 rounded-3xl p-8 backdrop-blur-xl hover:border-slate-700 transition-colors group"
    >
      <p className="text-[10px] font-black text-slate-500 tracking-widest mb-4 uppercase">{label}</p>
      <div className="flex items-end justify-between">
        <h4 className="text-4xl font-black text-white group-hover:scale-105 transition-transform origin-left">{value}</h4>
        <div className={`text-[10px] font-bold bg-slate-800 rounded-full px-3 py-1 ${color}`}>
          {trend}
        </div>
      </div>
    </motion.div>
  );
}

function ActivityItem({ icon, text, time }: any) {
  return (
    <div className="flex items-start gap-4">
      <div className="text-lg mt-0.5">{icon}</div>
      <div className="flex-1 min-w-0">
        <p className="text-sm text-slate-300 line-clamp-1">{text}</p>
        <p className="text-[10px] text-slate-500 font-medium uppercase tracking-tighter mt-1">{time}</p>
      </div>
    </div>
  );
}
