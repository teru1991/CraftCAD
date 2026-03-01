#include "fillet_tool.h"
#include "../ffi/craftcad_ffi.h"
#include "../hittest.h"
#include <QJsonDocument>
#include <QMessageBox>
static QString take(char* ptr){ if(!ptr) return {}; QString s=QString::fromUtf8(ptr); craftcad_free_string(ptr); return s; }
FilletTool::FilletTool(DocStore* s, Camera* c):store_(s),camera_(c){}
void FilletTool::onPointerDown(const QPointF& p){ last_=p; auto h=hitTest(*store_,*camera_,p,8.0); if(!h) return; if(a_.isEmpty()) a_=h->entityId; else if(b_.isEmpty()) b_=h->entityId; if(!a_.isEmpty()&&!b_.isEmpty()){ double r=numeric_.value().value_or(5.0); QByteArray doc=store_->documentJson().toUtf8(); QJsonObject in{{"e1",a_},{"e2",b_},{"radius",r}}; QByteArray ib=QJsonDocument(in).toJson(QJsonDocument::Compact), eb=store_->epsPolicyJson().toUtf8(); QString env=take(craftcad_history_apply_fillet(store_->historyHandle(),doc.constData(),ib.constData(),eb.constData())); auto root=QJsonDocument::fromJson(env.toUtf8()).object(); if(!root.value("ok").toBool()) QMessageBox::warning(nullptr,"Fillet",QString::fromUtf8(QJsonDocument(root.value("reason").toObject()).toJson())); else store_->setDocumentJson(QString::fromUtf8(QJsonDocument(root.value("data").toObject().value("document").toObject()).toJson(QJsonDocument::Compact))); a_.clear(); b_.clear(); }}
void FilletTool::onKeyPress(QKeyEvent* e){ if(e->key()==Qt::Key_Escape){a_.clear();b_.clear(); ambiguity_.clear();} if(e->key()==Qt::Key_Tab) ambiguity_.onTab(1); numeric_.handleKey(e->key(),e->text()); }
void FilletTool::onWheel(QWheelEvent* e){ ambiguity_.onWheel(e); }
void FilletTool::renderOverlay(QPainter& p){ ambiguity_.render(p,*camera_); }
