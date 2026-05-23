# NexusTask Enterprise v2.1

NexusTask es un sistema de gestión de tareas empresarial de alto rendimiento, re-implementado siguiendo los más altos estándares de ingeniería de software. Utiliza una **Arquitectura Hexagonal** pura para garantizar la mantenibilidad, escalabilidad y robustez del sistema.

## 🚀 Características Principales

- **Arquitectura Hexagonal**: Separación clara entre Dominio, Aplicación e Infraestructura.
- **Stack Moderno**:
  - **Backend**: Rust con Tauri para un rendimiento nativo y seguridad de memoria.
  - **Frontend**: React 18, TypeScript y Tailwind CSS para una experiencia de usuario premium.
  - **API REST**: Integración con Axum para servicios asíncronos de alta concurrencia.
- **Gestión Empresarial Avanzada**:
  - **Workspaces**: Aislamiento total de datos para equipos y organizaciones.
  - **Jerarquías Recursivas**: Implementación de *Nested Set* para manejar árboles de tareas infinitos de forma eficiente.
  - **Auditoría Unificada**: Registro detallado de todos los eventos del sistema.
  - **Campos Personalizados**: Flexibilidad total mediante almacenamiento JSON dinámico.
- **Seguridad**: Autenticación JWT y hashing de contraseñas con Argon2.

## 📁 Estructura del Proyecto

El proyecto se organiza siguiendo los principios de la arquitectura limpia:

- `src/domain`: Entidades de negocio y reglas puras.
- `src/application`: Casos de uso y servicios (lógica de coordinación).
- `src/ports`: Definición de interfaces (puertos) para adaptadores externos.
- `src/infrastructure`: Implementaciones concretas (SQLite, Axum, Tauri).
- `src-tauri/`: Punto de entrada del sistema y comandos de integración nativa.
- `frontend/` (o raíz `src/` web): Interfaz de usuario moderna basada en componentes React.

## 🛠️ Cómo Ejecutar

### Requisitos previos:
- Rust y Cargo (última versión estable)
- Node.js y pnpm

### Pasos para desarrollo:

1. **Instalar dependencias del frontend:**
   ```bash
   pnpm install
   ```

2. **Ejecutar en modo desarrollo (Tauri):**
   ```bash
   pnpm tauri dev
   ```

La aplicación abrirá una ventana nativa con la interfaz moderna y el servidor API se iniciará automáticamente en segundo plano.

## 📊 Estado del Proyecto

Para un análisis detallado del progreso y la cobertura técnica, consulta el [INFORME_ANALISIS.md](INFORME_ANALISIS.md).

---
**Desarrollado con ❤️ para NexusTask Enterprise**
