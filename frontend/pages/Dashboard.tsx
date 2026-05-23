import { motion } from 'framer-motion';

const stats = [
  { label: "TOTAL TAREAS", value: "1,284", trend: "+12% desde el mes pasado", color: "text-blue-400" },
  { label: "TASA COMPLETADO", value: "94.2%", trend: "+2.4% de mejora", color: "text-emerald-400" },
  { label: "RUTA CRÍTICA", value: "14", trend: "7 requieren atención", color: "text-amber-400" },
];

export default function Dashboard() {
  return (
    <div className="space-y-8">
      {/* Welcome Banner */}
      <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-blue-600 to-indigo-700 p-8 shadow-2xl shadow-blue-900/20">
        <div className="relative z-10">
          <h2 className="text-3xl font-bold text-white mb-2">¡Bienvenido de nuevo!</h2>
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
    </div>
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
