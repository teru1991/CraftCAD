#include "pattern_tool.h"
#include "../ffi/craftcad_ffi.h"
#include <QJsonDocument>
#include <QMessageBox>
static QString take(char* ptr){ if(!ptr) return {}; QString s=QString::fromUtf8(ptr); craftcad_free_string(ptr); return s; }
PatternTool::PatternTool(DocStore* s, Camera* c):store_(s),camera_(c){}
void PatternTool::onKeyPress(QKeyEvent* e){ numeric_.handleKey(e->key(),e->text()); if(e->key()==Qt::Key_Return||e->key()==Qt::Key_Enter){ int count=int(numeric_.value().value_or(3)); QJsonArray ids; for(const auto& id:store_->selection().ids()) ids.append(id); QJsonObject in{{"selection_ids",ids},{"params",QJsonObject{{"type","Linear"},{"dx",10.0},{"dy",0.0},{"count",count}}}}; QByteArray doc=store_->documentJson().toUtf8(), ib=QJsonDocument(in).toJson(QJsonDocument::Compact), eb=store_->epsPolicyJson().toUtf8(); QString env=take(craftcad_history_apply_pattern(store_->historyHandle(),doc.constData(),ib.constData(),eb.constData())); auto root=QJsonDocument::fromJson(env.toUtf8()).object(); if(!root.value("ok").toBool()) QMessageBox::warning(nullptr,"Pattern",QString::fromUtf8(QJsonDocument(root.value("reason").toObject()).toJson())); else store_->setDocumentJson(QString::fromUtf8(QJsonDocument(root.value("data").toObject().value("document").toObject()).toJson(QJsonDocument::Compact))); numeric_.clear(); }}
