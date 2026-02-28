#include <QApplication>
#include <QFileDialog>
#include <QGraphicsLineItem>
#include <QGraphicsScene>
#include <QGraphicsView>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QMainWindow>
#include <QMessageBox>
#include <QMouseEvent>
#include <QPen>
#include <QTransform>

extern "C" {
#include "craftcad_desktop_ffi.h"
}

class CadView final : public QGraphicsView {
public:
    explicit CadView(QWidget* parent = nullptr) : QGraphicsView(parent) {
        setRenderHint(QPainter::Antialiasing, true);
        setDragMode(QGraphicsView::NoDrag);
        setTransformationAnchor(QGraphicsView::AnchorUnderMouse);
        setResizeAnchor(QGraphicsView::AnchorViewCenter);
    }

protected:
    void wheelEvent(QWheelEvent* event) override {
        const qreal zoom = event->angleDelta().y() > 0 ? 1.15 : 1.0 / 1.15;
        scale(zoom, zoom);
    }

    void mousePressEvent(QMouseEvent* event) override {
        if (event->button() == Qt::MiddleButton) {
            _panning = true;
            _lastPos = event->pos();
            setCursor(Qt::ClosedHandCursor);
            event->accept();
            return;
        }
        QGraphicsView::mousePressEvent(event);
    }

    void mouseMoveEvent(QMouseEvent* event) override {
        if (_panning) {
            const QPointF delta = mapToScene(event->pos()) - mapToScene(_lastPos);
            translate(delta.x(), delta.y());
            _lastPos = event->pos();
            event->accept();
            return;
        }
        QGraphicsView::mouseMoveEvent(event);
    }

    void mouseReleaseEvent(QMouseEvent* event) override {
        if (event->button() == Qt::MiddleButton) {
            _panning = false;
            unsetCursor();
            event->accept();
            return;
        }
        QGraphicsView::mouseReleaseEvent(event);
    }

private:
    bool _panning = false;
    QPoint _lastPos;
};

static QString takeCString(char* ptr) {
    if (ptr == nullptr) {
        return QString();
    }
    const QString s = QString::fromUtf8(ptr);
    craftcad_desktop_string_free(ptr);
    return s;
}

static QString choosePath(const QStringList& args) {
    if (args.size() > 1) {
        return args[1];
    }
    return QFileDialog::getOpenFileName(nullptr, "Open .diycad", QString(), "DIYCAD Files (*.diycad)");
}

int main(int argc, char* argv[]) {
    QApplication app(argc, argv);

    const QString path = choosePath(app.arguments());
    if (path.isEmpty()) {
        return 0;
    }

    const QByteArray utf8Path = path.toUtf8();
    char* docPtr = craftcad_desktop_load_diycad_json(utf8Path.constData());
    if (docPtr == nullptr) {
        const QString err = takeCString(craftcad_desktop_last_error_json());
        QMessageBox::critical(nullptr, "Failed to open project", err);
        return 1;
    }

    const QString docJson = takeCString(docPtr);
    const QJsonDocument doc = QJsonDocument::fromJson(docJson.toUtf8());
    if (!doc.isObject()) {
        QMessageBox::critical(nullptr, "Invalid document", "Document JSON is not an object.");
        return 1;
    }

    auto* scene = new QGraphicsScene();
    scene->setBackgroundBrush(QColor(24, 24, 24));

    const QJsonArray entities = doc.object().value("entities").toArray();
    const QPen pen(QColor(0, 220, 255), 0.0);
    for (const QJsonValue& value : entities) {
        const QJsonObject entity = value.toObject();
        const QJsonObject geom = entity.value("geom").toObject();
        if (geom.value("type").toString() != "Line") {
            continue;
        }

        const QJsonObject a = geom.value("a").toObject();
        const QJsonObject b = geom.value("b").toObject();
        const double ax = a.value("x").toDouble();
        const double ay = -a.value("y").toDouble();
        const double bx = b.value("x").toDouble();
        const double by = -b.value("y").toDouble();
        auto* item = scene->addLine(QLineF(ax, ay, bx, by), pen);
        item->setZValue(10.0);
    }

    auto* view = new CadView();
    view->setScene(scene);
    view->setSceneRect(scene->itemsBoundingRect().adjusted(-50, -50, 50, 50));
    view->fitInView(scene->sceneRect(), Qt::KeepAspectRatio);

    QMainWindow window;
    window.setWindowTitle("CraftCAD Desktop (read-only)");
    window.setCentralWidget(view);
    window.resize(1024, 768);
    window.show();

    return app.exec();
}
