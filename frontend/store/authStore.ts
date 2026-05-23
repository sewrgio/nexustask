import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { User } from '../lib/api';

interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
  
  login: (username: string, password: string) => Promise<void>;
  register: (username: string, email: string, password: string) => Promise<void>;
  logout: () => void;
  clearError: () => void;
  setUser: (user: User) => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      token: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,

      login: async (username: string, password: string) => {
        set({ isLoading: true, error: null });
        try {
          const { authAPI } = await import('../lib/api');
          const response = await authAPI.login(username, password);
          const { token, user } = response.data;
          
          localStorage.setItem('jwt_token', token);
          set({ user, token, isAuthenticated: true, isLoading: false });
        } catch (error: any) {
          const errorMessage = error.response?.data?.message || 'Error al iniciar sesión';
          set({ error: errorMessage, isLoading: false });
          throw error;
        }
      },

      register: async (username: string, email: string, password: string) => {
        set({ isLoading: true, error: null });
        try {
          const { authAPI } = await import('../lib/api');
          const response = await authAPI.register(username, email, password);
          const { token, user } = response.data;
          
          localStorage.setItem('jwt_token', token);
          set({ user, token, isAuthenticated: true, isLoading: false });
        } catch (error: any) {
          const errorMessage = error.response?.data?.message || 'Error al registrarse';
          set({ error: errorMessage, isLoading: false });
          throw error;
        }
      },

      logout: () => {
        localStorage.removeItem('jwt_token');
        set({ user: null, token: null, isAuthenticated: false });
      },

      clearError: () => set({ error: null }),
      
      setUser: (user: User) => set({ user }),
    }),
    {
      name: 'auth-storage',
      partialize: (state) => ({ user: state.user, token: state.token, isAuthenticated: state.isAuthenticated }),
    }
  )
);
