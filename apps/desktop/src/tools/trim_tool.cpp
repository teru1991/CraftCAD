#include "trim_tool.h"
#include "../ffi/craftcad_ffi.h"
#include "../hittest.h"
#include <QJsonDocument>
#include <QMessageBox>

static QString take_trim(char* ptr){ if(!ptr) return {}; QString s=QString::fromUtf8(ptr); craftcad_free_string(ptr); return s; }

TrimTool::TrimTool(DocStore* store, Camera* camera) : store_(store), camera_(camera) {}

void TrimTool::onPointerDown(const QPointF& p) {
    lastScreen_ = p;
    auto hit = hitTest(*store_, *camera_, p, 8.0);
    if (!hit) return;
    if (step_ == Step::PickTarget) {
        targetId_ = hit->entityId;
        store_->selection().setSingle(targetId_);
        step_ = Step::PickCutter;
    } else if (step_ == Step::PickCutter) {
        cutterId_ = hit->entityId;
        if (cutterId_ == targetId_) return;
        QString reason;
        runTrim(&reason, false);
    }
}

void TrimTool::onPointerMove(const QPointF& p) {
    lastScreen_ = p;
}

bool TrimTool::runTrim(QString* reason, bool commit, int candidateIndex) {
    if (targetId_.isEmpty() || cutterId_.isEmpty()) return false;
    QByteArray doc = store_->documentJson().toUtf8();
    QByteArray tid = targetId_.toUtf8();
    QByteArray cid = cutterId_.toUtf8();
    auto wp = camera_->screenToWorld(lastScreen_);
    QByteArray pick = QJsonDocument(QJsonObject{{"x",wp.x},{"y",wp.y}}).toJson(QJsonDocument::Compact);
    QByteArray eps = store_->epsPolicyJson().toUtf8();

    QString env = take_trim(
        commit
            ? craftcad_history_apply_trim_entity_with_candidate_index(store_->historyHandle(), doc.constData(), tid.constData(), cid.constData(), pick.constData(), eps.constData(), candidateIndex)
            : craftcad_history_apply_trim_entity(store_->historyHandle(), doc.constData(), tid.constData(), cid.constData(), pick.constData(), eps.constData())
    );

    auto root = QJsonDocument::fromJson(env.toUtf8()).object();
    if (!root.value("ok").toBool()) {
        QJsonObject r = root.value("reason").toObject();
        if (r.value("code").toString() == "EDIT_TRIM_AMBIGUOUS_CANDIDATE") {
            ambiguity_.setCandidates(r.value("debug").toObject().value("candidates").toArray());
            step_ = Step::PreviewOrAmbiguous;
        }
        if (reason) *reason = QString::fromUtf8(QJsonDocument(r).toJson());
        return false;
    }
    QString newDoc = QString::fromUtf8(QJsonDocument(root.value("data").toObject().value("document").toObject()).toJson(QJsonDocument::Compact));
    if (commit) store_->setDocumentJson(newDoc);
    step_ = Step::PickTarget;
    targetId_.clear();
    cutterId_.clear();
    ambiguity_.clear();
    return true;
}

void TrimTool::onPointerUp(const QPointF&) {
    if (step_ == Step::PreviewOrAmbiguous) return;
}

void TrimTool::cycleCandidate(int delta) {
    ambiguity_.onTab(delta);
}

void TrimTool::onKeyPress(QKeyEvent* e) {
    if (e->key() == Qt::Key_Escape) { step_ = Step::PickTarget; targetId_.clear(); cutterId_.clear(); ambiguity_.clear(); return; }
    if (step_ == Step::PreviewOrAmbiguous) {
        if (e->key() == Qt::Key_Tab) cycleCandidate(1);
        if (e->key() == Qt::Key_Backtab) cycleCandidate(-1);
        if (e->key() == Qt::Key_Return || e->key() == Qt::Key_Enter) {
            QString reason;
            if (!runTrim(&reason, true, ambiguity_.currentIndex())) QMessageBox::warning(nullptr, "Trim failed", reason);
        }
    }
}

void TrimTool::renderOverlay(QPainter& p) {
    ambiguity_.render(p, *camera_);
}

void TrimTool::onWheel(QWheelEvent* e) {
    if (step_ != Step::PreviewOrAmbiguous || !ambiguity_.active()) return;
    ambiguity_.onWheel(e);
}
