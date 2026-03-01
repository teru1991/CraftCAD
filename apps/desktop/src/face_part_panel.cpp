#include "face_part_panel.h"
#include "canvas_widget.h"
#include "ffi/craftcad_ffi.h"
#include <QInputDialog>
#include <QJsonDocument>
#include <QListWidget>
#include <QMessageBox>
#include <QPushButton>
#include <QVBoxLayout>
#include <QUuid>

static QString take_face(char* ptr){ if(!ptr) return {}; QString s=QString::fromUtf8(ptr); craftcad_free_string(ptr); return s; }

FacePartPanel::FacePartPanel(DocStore* store, CanvasWidget* canvas, QWidget* parent)
    : QWidget(parent), store_(store), canvas_(canvas) {
    auto* lay = new QVBoxLayout(this);
    auto* detect = new QPushButton("Detect Faces", this);
    auto* create = new QPushButton("Create Part from Selected Face", this);
    list_ = new QListWidget(this);
    lay->addWidget(detect);
    lay->addWidget(list_);
    lay->addWidget(create);
    connect(detect, &QPushButton::clicked, [this]{ detectFaces(); });
    connect(create, &QPushButton::clicked, [this]{ createPart(); });
    connect(list_, &QListWidget::currentRowChanged, [this](int){ onFaceSelectionChanged(); });
}

void FacePartPanel::detectFaces() {
    QByteArray doc = store_->documentJson().toUtf8();
    QByteArray eps = store_->epsPolicyJson().toUtf8();
    QString env = take_face(craftcad_extract_faces(doc.constData(), eps.constData()));
    auto root = QJsonDocument::fromJson(env.toUtf8()).object();
    if (!root.value("ok").toBool()) {
        QMessageBox::warning(this, "Detect Faces failed", QString::fromUtf8(QJsonDocument(root.value("reason").toObject()).toJson()));
        return;
    }
    faces_ = root.value("data").toObject().value("faces").toArray();
    list_->clear();
    for (int i = 0; i < faces_.size(); ++i) {
        auto f = faces_[i].toObject();
        list_->addItem(QString("Face %1 (holes: %2)").arg(i + 1).arg(f.value("holes").toArray().size()));
    }
    if (!faces_.isEmpty()) list_->setCurrentRow(0);
}

void FacePartPanel::onFaceSelectionChanged() {
    int row = list_->currentRow();
    if (row < 0 || row >= faces_.size()) {
        canvas_->setHighlightedFace(QJsonObject{});
        return;
    }
    canvas_->setHighlightedFace(faces_[row].toObject());
}

void FacePartPanel::createPart() {
    int row = list_->currentRow();
    if (row < 0 || row >= faces_.size()) {
        QMessageBox::warning(this, "Create Part", "FACE_NO_CLOSED_LOOP");
        return;
    }
    bool ok = false;
    QString name = QInputDialog::getText(this, "Part", "Name", QLineEdit::Normal, "Part", &ok);
    if (!ok || name.isEmpty()) return;
    double thickness = QInputDialog::getDouble(this, "Part", "Thickness", 18.0, 0.0, 1e9, 3, &ok);
    if (!ok) return;
    int qty = QInputDialog::getInt(this, "Part", "Quantity", 1, 1, 1000000, 1, &ok);
    if (!ok) return;
    QString materialId = QInputDialog::getText(this, "Part", "Material UUID", QLineEdit::Normal, QUuid::createUuid().toString(QUuid::WithoutBraces), &ok);
    if (!ok || materialId.isEmpty()) return;
    double margin = QInputDialog::getDouble(this, "Part", "Margin", 0.0, 0.0, 1e6, 3, &ok);
    if (!ok) return;
    double kerf = QInputDialog::getDouble(this, "Part", "Kerf", 0.0, 0.0, 1e6, 3, &ok);
    if (!ok) return;

    QJsonObject face = faces_[row].toObject();
    QJsonObject part{
        {"id", QUuid::createUuid().toString(QUuid::WithoutBraces)},
        {"name", name},
        {"outline", QJsonObject{{"outer", face.value("outer").toArray()}, {"holes", face.value("holes").toArray()}}},
        {"thickness", thickness},
        {"quantity", qty},
        {"material_id", materialId},
        {"grain_dir", QJsonValue::Null},
        {"allow_rotate", true},
        {"margin", margin},
        {"kerf", kerf}
    };

    QByteArray doc = store_->documentJson().toUtf8();
    QByteArray pb = QJsonDocument(part).toJson(QJsonDocument::Compact);
    QString env = take_face(craftcad_history_apply_create_part(store_->historyHandle(), doc.constData(), pb.constData()));
    auto root = QJsonDocument::fromJson(env.toUtf8()).object();
    if (!root.value("ok").toBool()) {
        QMessageBox::warning(this, "Create Part failed", QString::fromUtf8(QJsonDocument(root.value("reason").toObject()).toJson()));
        return;
    }
    QString newDoc = QString::fromUtf8(QJsonDocument(root.value("data").toObject().value("document").toObject()).toJson(QJsonDocument::Compact));
    store_->setDocumentJson(newDoc);
    QMessageBox::information(this, "Part", "Part created.");
}
