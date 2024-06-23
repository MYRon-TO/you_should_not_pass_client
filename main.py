import gi
gi.require_version('Gtk', '3.0')
from gi.repository import Gtk, Gio,Gdk

class MainWindow(Gtk.Window):
    def __init__(self, **kargs):
        super().__init__(**kargs, title='用户名密码管理软件')
        self.set_default_size(500, 300)

        self.vbox = Gtk.VBox(spacing=6)
        self.vbox.set_border_width(6)
        self.add(self.vbox)  # 添加 vbox 到窗口

        self.liststore = Gtk.ListStore(str,bool,str)  # 存储用户名，选中状态和密码
        self.treeview = Gtk.TreeView(model=self.liststore)

        self.scrolled_window = Gtk.ScrolledWindow()
        self.scrolled_window.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        self.scrolled_window.add(self.treeview)  # 将 treeview 添加到 scrolled_window
        self.vbox.pack_start(self.scrolled_window, True, True, 0)  # 添加 scrolled_window 到 vbox

        # 填充数据
        for i in range(0, 50):
            self.liststore.append(['网站名{}'.format(i), None, '密码{}'.format(i)])
        cellrenderer_toggle = Gtk.CellRendererToggle()
        cellrenderer_text = Gtk.CellRendererText()
        cellrenderer_text.set_property('editable', True)
        column_toggle = Gtk.TreeViewColumn('选中', cellrenderer_toggle, active=1)
        column_text = Gtk.TreeViewColumn('网站名', cellrenderer_text, text=0)
        column_toggle.set_clickable(True)  # 确保列可点击
        # 连接 toggled 信号到 on_toggle_toggled 方法
        cellrenderer_toggle.connect("toggled", self.on_toggle_toggled)
        self.treeview.append_column(column_text)
        self.treeview.append_column(column_toggle)

# 连接 row-activated 信号
        self.treeview.connect("row-activated", self.on_row_activated)

        self.add_button = Gtk.Button(label="增加")
        self.add_button.connect("clicked", self.on_add_clicked)
        self.vbox.pack_start(self.add_button, False, False, 0)

        self.remove_button = Gtk.Button(label="删除")
        self.remove_button.connect("clicked", self.on_remove_clicked)
        self.vbox.pack_start(self.remove_button, False, False, 0)
    def on_add_clicked(self, button):
        # 添加新项到列表
        self.liststore.append(['新网站名', False, '新密码'])
    
    def on_remove_clicked(self, button):
    # 获取模型
        model = self.treeview.get_model()
    # 存储所有被勾选的行的路径
        selected_paths = []

    # 遍历模型中的所有行
        iter = model.get_iter_first()
        while iter:
        # 检查复选框列的值
            if model.get_value(iter, 1):  # 假设第二列是复选框状态
                # 如果复选框被勾选，添加路径到列表
                selected_paths.append(model.get_path(iter))
        # 移动到下一行
            iter = model.iter_next(iter)

    # 逆序遍历路径列表，以便安全地删除项
        for path in reversed(selected_paths):
        # 获取迭代器并删除行
            iter = model.get_iter(path)
            model.remove(iter)

    # 恢复滚动条位置
    def on_row_activated(self, treeview, path, view_column):
        # 获取模型中的数据
        model = treeview.get_model()
        iter = model.get_iter(path)
        username = model.get_value(iter, 0)
        password = model.get_value(iter, 2)

        # 创建一个带有超链接和下方文本的弹窗
        dialog = Gtk.Dialog(
            "网站名",
            transient_for=self,
            flags=0,
        )
        dialog.add_buttons(Gtk.STOCK_OK, Gtk.ResponseType.OK)
        # 创建一个标签，并设置其 markup 来包含超链接和下方文本
        label = Gtk.Label()
        label.set_use_markup(True)
        label.set_line_wrap(True)
        hyperlink = f'<span foreground="blue" underline="single">网址: <a href="<url id="" type="url" status="" title="" wc="">http://www.baidu.com">www.baidu.com</a></span>'
        user_pass_text = f"\n用户名: {username}\n密码: {password}"
        label.set_markup(f"{hyperlink}{user_pass_text}")

        dialog.get_content_area().add(label)
        dialog.show_all()

        # 显示弹窗并等待用户响应
        response = dialog.run()
        dialog.destroy()

    def on_toggle_toggled(self, cell_renderer, path):
    # 获取模型
        model = self.treeview.get_model()
    # 根据 path 获取迭代器
        iter = model.get_iter(path)
    # 获取当前选中状态
        current_state = model.get_value(iter, 1)
    # 切换状态
        new_state = not current_state
    # 更新模型
        model.set_value(iter, 1, new_state)
    # 打印出路径和新状态
        print(f"Toggled {path} to {new_state}")
def on_activate(app):
    win = MainWindow(application=app)
    win.connect("destroy", Gtk.main_quit)
    win.show_all()

app = Gtk.Application.new('org.gtk.example', Gio.ApplicationFlags.FLAGS_NONE)
app.connect('activate', on_activate)

if __name__ == "__main__":
    win = MainWindow()
    win.connect("destroy", Gtk.main_quit)
    win.show_all()
    Gtk.main()