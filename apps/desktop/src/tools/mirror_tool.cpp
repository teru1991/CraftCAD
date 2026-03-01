#include "mirror_tool.h"
#include "../ffi/craftcad_ffi.h"
#include <QJsonDocument>
#include <QMessageBox>
static QString take(char* ptr){ if(!ptr) return {}; QString s=QString::fromUtf8(ptr); craftcad_free_string(ptr); return s; }
MirrorTool::MirrorTool(DocStore* s, Camera* c):store_(s),camera_(c){}
void MirrorTool::onPointerDown(const QPointF& p){ WVec2 w=camera_->screenToWorld(p); if(!haveA_){a_=w;haveA_=true; return;} b_=w; haveA_=false; QJsonArray ids; for(const auto& id:store_->selection().ids()) ids.append(id); QJsonObject in{{"selection_ids",ids},{"axis_a",QJsonObject{{"x",a_.x},{"y",a_.y}}},{"axis_b",QJsonObject{{"x",b_.x},{"y",b_.y}}}}; QByteArray doc=store_->documentJson().toUtf8(), ib=QJsonDocument(in).toJson(QJsonDocument::Compact), eb=store_->epsPolicyJson().toUtf8(); QString env=take(craftcad_history_apply_mirror(store_->historyHandle(),doc.constData(),ib.constData(),eb.constData())); auto root=QJsonDocument::fromJson(env.toUtf8()).object(); if(!root.value("ok").toBool()) QMessageBox::warning(nullptr,"Mirror",QString::fromUtf8(QJsonDocument(root.value("reason").toObject()).toJson())); else store_->setDocumentJson(QString::fromUtf8(QJsonDocument(root.value("data").toObject().value("document").toObject()).toJson(QJsonDocument::Compact))); }
void MirrorTool::onKeyPress(QKeyEvent* e){ if(e->key()==Qt::Key_Escape) haveA_=false; }
