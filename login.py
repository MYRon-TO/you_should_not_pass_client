import gi

gi.require_version('Gtk', '4.0')
from gi.repository import Gtk


class ButtonWindow(Gtk.ApplicationWindow):
    def __init__(self, **kargs):
        super().__init__(**kargs, title='用户名密码管理软件')

        self.set_size_request(400, 300)

        vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
        vbox.props.margin_start = 24
        vbox.props.margin_end = 24
        vbox.props.margin_top = 24
        vbox.props.margin_bottom = 24
        self.set_child(vbox)

        pass_entry = Gtk.PasswordEntry()
        pass_entry.props.placeholder_text = 'Password Entry'
        pass_entry.props.show_peek_icon = True
        pass_entry.props.margin_top = 24
        vbox.append(pass_entry)

        button = Gtk.Button.new_with_mnemonic('登录')
        button.connect('clicked', self.on_close_clicked)
        vbox.append(button)

    def on_close_clicked(self, _button):
        print('login application')
        self.close()


def on_activate(app):
    win = ButtonWindow(application=app)
    win.present()


app = Gtk.Application(application_id='com.example.App')
app.connect('activate', on_activate)

app.run(None)