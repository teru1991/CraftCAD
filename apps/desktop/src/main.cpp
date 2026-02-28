#include "canvas_widget.h"
#include "doc_store.h"
#include <QApplication>
#include <QFileDialog>
#include <QMainWindow>
#include <QMessageBox>

int main(int argc, char* argv[]) {
    QApplication app(argc, argv);

    QString path;
    if (app.arguments().size() > 1) path = app.arguments()[1];
    if (path.isEmpty()) path = QFileDialog::getOpenFileName(nullptr, "Open .diycad", QString(), "DIYCAD Files (*.diycad)");
    if (path.isEmpty()) return 0;

    DocStore store;
    QString err;
    if (!store.loadDiycad(path, &err)) {
        QMessageBox::critical(nullptr, "Failed to open project", err);
        return 1;
    }

    QMainWindow w;
    w.setWindowTitle("CraftCAD Desktop (line/move/rotate/scale)");
    w.setCentralWidget(new CanvasWidget(&store));
    w.resize(1200, 800);
    w.show();
    return app.exec();
}
