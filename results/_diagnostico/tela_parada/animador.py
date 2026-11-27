"""Janela pequena que redesenha sem parar, sempre por cima e sem receber foco (canto superior direito)."""
import ctypes, tkinter as tk
r = tk.Tk()
r.withdraw()
r.overrideredirect(True)
r.geometry(f"90x90+{r.winfo_screenwidth() - 100}+10")
c = tk.Canvas(r, width=90, height=90, bg="black", highlightthickness=0)
c.pack()
arco = c.create_arc(10, 10, 80, 80, start=0, extent=100, style="arc", outline="white", width=6)
r.update_idletasks()
u = ctypes.windll.user32
h = u.GetParent(r.winfo_id()) or r.winfo_id()
u.SetWindowLongW(h, -20, u.GetWindowLongW(h, -20) | 0x08000000 | 0x80 | 0x8)  # NOACTIVATE | TOOLWINDOW | TOPMOST
u.ShowWindow(h, 4)  # mostrar sem ativar
u.SetWindowPos(h, -1, 0, 0, 0, 0, 0x13)
a = 0
def passo():
    global a
    a = (a + 6) % 360
    c.itemconfig(arco, start=a)
    r.after(8, passo)
passo()
u.ShowWindow(h, 4)
r.after(1000, lambda: print('animador visivel', u.IsWindowVisible(h), 'quadros ~', a // 6, flush=True))
r.mainloop()
