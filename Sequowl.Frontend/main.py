import sys
from unittest.util import three_way_cmp

from PySide6 import QtWidgets
from PySide6.QtWidgets import QTreeWidget, QApplication, QTreeWidgetItem

data = {"Project A": ["file_a.py", "file_a.txt", "something.xls"],
        "Project B": ["file_b.csv", "photo.jpg"],
        "Project C": []}



if __name__ == '__main__':
    app = QApplication([])

    tree = QTreeWidget()
    tree.setColumnCount(2)
    tree.setHeaderLabels(["Name", "Type"])

    items = []
    for key, values in data.items():
        item = QTreeWidgetItem([key])
        for value in values:
            ext = value.split(".")[-1].upper()
            child = QTreeWidgetItem([value])
            item.addChild(child)
        items.append(item)

    tree.insertTopLevelItems(0, items)
    tree.show()



    sys.exit(app.exec())

# class MainWidget(QtWidgets.QWidget):
#     def __init__(self):
#         super(MainWidget, self).__init__()
#         self.setWindowTitle("My app")
#         self.setWindowIcon(QtGui.QIcon('icon.png'))
#         self.layout = QtWidgets.QVBoxLayout(self)
#         button = QPushButton("Click me")
#         button.show()
#         button.clicked.connect(say_hello)
#         # typeable line
#         line_edit = QLineEdit()
#         self.layout.addWidget(button)
#         self.layout.addWidget(line_edit)


# Slot is the action that is triggered when something is activated.
# The action itself is called Signal.
# @Slot()
# class Form(QDialog):
#     def greetings(self):
#         print(f"hello {self.edit.text()}")
#
#     def __init__(self, parent=None):
#         super(Form, self).__init__(parent)
#         self.setWindowTitle("My Form")
#         self.edit = QLineEdit("Your name here!")
#         self.button = QPushButton("Say hello!")
#         layout = QVBoxLayout()
#         # layout.setContentsMargins(0, 0, 0, 0)
#         # layout.setSpacing(0)
#
#         layout.addWidget(self.edit)
#         layout.addWidget(self.button)
#
#         self.setLayout(layout)
#         self.button.clicked.connect(self.greetings)
#
#
# colors = [("Red", "#FF0000"),
#           ("Green", "#00FF00"),
#           ("Blue", "#0000FF"),
#           ("Black", "#000000"),
#           ("White", "#FFFFFF"),
#           ("Electric Green", "#41CD52"),
#           ("Dark Blue", "#222840"),
#           ("Yellow", "#F9E56d")]
#
#
# def get_rgb_from_hex(code):
#     code_hex = code.replace("#", "")
#     rgb = tuple(int(code_hex[i:i + 2], 16) for i in (0, 2, 4))
#     return QColor.fromRgb(rgb[0], rgb[1], rgb[2])

# table = QTableWidget()
# table.setColumnCount(len(colors))
# table.setRowCount(len(colors[0]) + 1)
# table.setHorizontalHeaderLabels(["Name", "Hex Code", "Color"])
#
# for i, (name, code) in enumerate(colors):
#     item_name = QTableWidgetItem(name)
#     item_code = QTableWidgetItem(code)
#     item_color = QTableWidgetItem()
#     item_color.setBackground(get_rgb_from_hex(code))
#     table.setItem(i, 0, item_name)
#     table.setItem(i, 1, item_code)
#     table.setItem(i, 2, item_color)
#
# table.show()
# form = Form()
# form.show()
# widget = MainWidget()
# widget = QWidget()
# widget.resize(900, 800)
# widget.show()
