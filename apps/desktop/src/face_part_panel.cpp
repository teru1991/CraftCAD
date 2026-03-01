#include "face_part_panel.h"
#include "canvas_widget.h"
#include "ffi/craftcad_ffi.h"
#include <QFileDialog>
#include <QIODevice>
#include <QFile>
#include <QInputDialog>
#include <QJsonDocument>
#include <QJsonObject>
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
    auto* refresh = new QPushButton("Refresh Parts", this);
    auto* edit = new QPushButton("Edit Selected Part", this);
    auto* del = new QPushButton("Delete Selected Part", this);
    auto* bom = new QPushButton("Export BOM", this);
    facesList_ = new QListWidget(this);
    partsList_ = new QListWidget(this);
    lay->addWidget(detect);
    lay->addWidget(facesList_);
    lay->addWidget(create);
    lay->addWidget(refresh);
    lay->addWidget(partsList_);
    lay->addWidget(edit);
    lay->addWidget(del);
    lay->addWidget(bom);
    connect(detect, &QPushButton::clicked, [this]{ detectFaces(); });
    connect(create, &QPushButton::clicked, [this]{ createPartFromFace(); });
    connect(refresh, &QPushButton::clicked, [this]{ refreshParts(); });
    connect(edit, &QPushButton::clicked, [this]{ editSelectedPart(); });
    connect(del, &QPushButton::clicked, [this]{ deleteSelectedPart(); });
    connect(bom, &QPushButton::clicked, [this]{ exportBom(); });
    connect(facesList_, &QListWidget::currentRowChanged, [this](int){ onFaceSelectionChanged(); });
    refreshParts();
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
    facesList_->clear();
    for (int i = 0; i < faces_.size(); ++i) {
        auto f = faces_[i].toObject();
        facesList_->addItem(QString("Face %1 (holes: %2)").arg(i + 1).arg(f.value("holes").toArray().size()));
    }
    if (!faces_.isEmpty()) facesList_->setCurrentRow(0);
}

void FacePartPanel::onFaceSelectionChanged() {
    int row = facesList_->currentRow();
    if (row < 0 || row >= faces_.size()) {
        canvas_->setHighlightedFace(QJsonObject{});
        return;
    }
    canvas_->setHighlightedFace(faces_[row].toObject());
}

void FacePartPanel::createPartFromFace() {
    int row = facesList_->currentRow();
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

    QJsonObject docObj = QJsonDocument::fromJson(store_->documentJson().toUtf8()).object();
    auto mats = docObj.value("materials").toArray();
    if (mats.isEmpty()) { QMessageBox::warning(this, "Create Part", "MATERIAL_NOT_FOUND"); return; }
    QString materialId = mats[0].toObject().value("id").toString();

    double margin = QInputDialog::getDouble(this, "Part", "Margin", 0.0, 0.0, 1e6, 3, &ok);
    if (!ok) return;
    double kerf = QInputDialog::getDouble(this, "Part", "Kerf", 0.0, 0.0, 1e6, 3, &ok);
    if (!ok) return;

    QJsonObject face = faces_[row].toObject();
    QJsonObject props{{"name",name},{"thickness",thickness},{"quantity",qty},{"material_id",materialId},{"grain_dir",QJsonValue::Null},{"allow_rotate",true},{"margin",margin},{"kerf",kerf}};
    QByteArray doc = store_->documentJson().toUtf8();
    QByteArray fb = QJsonDocument(face).toJson(QJsonDocument::Compact);
    QByteArray pb = QJsonDocument(props).toJson(QJsonDocument::Compact);
    QString env = take_face(craftcad_history_apply_create_part_from_face(store_->historyHandle(), doc.constData(), fb.constData(), pb.constData()));
    auto root = QJsonDocument::fromJson(env.toUtf8()).object();
    if (!root.value("ok").toBool()) {
        QMessageBox::warning(this, "Create Part failed", QString::fromUtf8(QJsonDocument(root.value("reason").toObject()).toJson()));
        return;
    }
    QString newDoc = QString::fromUtf8(QJsonDocument(root.value("data").toObject().value("document").toObject()).toJson(QJsonDocument::Compact));
    store_->setDocumentJson(newDoc);
    refreshParts();
}

void FacePartPanel::refreshParts() {
    partsList_->clear();
    QJsonObject docObj = QJsonDocument::fromJson(store_->documentJson().toUtf8()).object();
    for (const auto& pv : docObj.value("parts").toArray()) {
        auto p = pv.toObject();
        auto* item = new QListWidgetItem(QString("%1 | qty=%2").arg(p.value("name").toString()).arg(p.value("quantity").toInt()));
        item->setData(Qt::UserRole, p.value("id").toString());
        partsList_->addItem(item);
    }
}

void FacePartPanel::editSelectedPart() {
    auto* item = partsList_->currentItem();
    if (!item) return;
    QString pid = item->data(Qt::UserRole).toString();
    bool ok = false;
    QString name = QInputDialog::getText(this, "Edit Part", "Name", QLineEdit::Normal, item->text().split('|').first().trimmed(), &ok);
    if (!ok || name.isEmpty()) return;
    int qty = QInputDialog::getInt(this, "Edit Part", "Quantity", 1, 1, 1000000, 1, &ok);
    if (!ok) return;
    QJsonObject patch{{"name",name},{"quantity",qty}};
    QByteArray doc = store_->documentJson().toUtf8();
    QByteArray id = pid.toUtf8();
    QByteArray pb = QJsonDocument(patch).toJson(QJsonDocument::Compact);
    QString env = take_face(craftcad_history_apply_update_part(store_->historyHandle(), doc.constData(), id.constData(), pb.constData()));
    auto root = QJsonDocument::fromJson(env.toUtf8()).object();
    if (!root.value("ok").toBool()) { QMessageBox::warning(this,"Update Part failed", QString::fromUtf8(QJsonDocument(root.value("reason").toObject()).toJson())); return; }
    QString newDoc = QString::fromUtf8(QJsonDocument(root.value("data").toObject().value("document").toObject()).toJson(QJsonDocument::Compact));
    store_->setDocumentJson(newDoc);
    refreshParts();
}

void FacePartPanel::deleteSelectedPart() {
    auto* item = partsList_->currentItem();
    if (!item) return;
    QByteArray doc = store_->documentJson().toUtf8();
    QByteArray id = item->data(Qt::UserRole).toString().toUtf8();
    QString env = take_face(craftcad_history_apply_delete_part(store_->historyHandle(), doc.constData(), id.constData()));
    auto root = QJsonDocument::fromJson(env.toUtf8()).object();
    if (!root.value("ok").toBool()) { QMessageBox::warning(this,"Delete Part failed", QString::fromUtf8(QJsonDocument(root.value("reason").toObject()).toJson())); return; }
    QString newDoc = QString::fromUtf8(QJsonDocument(root.value("data").toObject().value("document").toObject()).toJson(QJsonDocument::Compact));
    store_->setDocumentJson(newDoc);
    refreshParts();
}

void FacePartPanel::exportBom() {
    QByteArray doc = store_->documentJson().toUtf8();
    QByteArray opts = QJsonDocument(QJsonObject{{"delimiter",","}}).toJson(QJsonDocument::Compact);
    QString env = take_face(craftcad_export_bom_csv_bytes(doc.constData(), opts.constData()));
    auto root = QJsonDocument::fromJson(env.toUtf8()).object();
    if (!root.value("ok").toBool()) { QMessageBox::warning(this,"Export BOM failed", QString::fromUtf8(QJsonDocument(root.value("reason").toObject()).toJson())); return; }
    auto d = root.value("data").toObject();
    QByteArray bytes = QByteArray::fromBase64(d.value("bytes_base64").toString().toUtf8());
    QString path = QFileDialog::getSaveFileName(this, "Save BOM", d.value("filename").toString(), "CSV (*.csv)");
    if (path.isEmpty()) return;
    QFile f(path);
    if (!f.open(QIODevice::WriteOnly)) { QMessageBox::warning(this,"Export BOM failed","BOM_EXPORT_FAILED"); return; }
    f.write(bytes);
    f.close();
    QMessageBox::information(this, "Export BOM", "BOM exported.");
}
