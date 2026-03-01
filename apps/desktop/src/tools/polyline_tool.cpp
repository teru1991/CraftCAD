#include "polyline_tool.h"
#include "../ffi/craftcad_ffi.h"
#include <QJsonDocument>
#include <QMessageBox>
#include <cmath>
static QString take(char* ptr){ if(!ptr) return {}; QString s=QString::fromUtf8(ptr); craftcad_free_string(ptr); return s; }
PolylineTool::PolylineTool(DocStore* store, Camera* camera):store_(store),camera_(camera){}
void PolylineTool::onPointerDown(const QPointF& s){ WVec2 w=camera_->screenToWorld(s); snap_=computeSnap(*store_,w,pts_.isEmpty()?std::nullopt:std::optional<WVec2>(pts_.last())); WVec2 p=snap_.best?snap_.best->point:w; if(!pts_.isEmpty()){ if(lockH) p.y=pts_.last().y; if(lockV) p.x=pts_.last().x; if(auto n=numeric_.value()){ double dx=p.x-pts_.last().x, dy=p.y-pts_.last().y; double d=std::sqrt(dx*dx+dy*dy); if(d>0){ p={pts_.last().x+dx/d*(*n),pts_.last().y+dy/d*(*n)}; } } }
pts_.push_back(p); hover_.reset(); numeric_.clear(); }
void PolylineTool::onPointerMove(const QPointF& s){ if(pts_.isEmpty()) return; WVec2 w=camera_->screenToWorld(s); snap_=computeSnap(*store_,w,pts_.last()); hover_=snap_.best?snap_.best->point:w; if(lockH) hover_->y=pts_.last().y; if(lockV) hover_->x=pts_.last().x; }
void PolylineTool::commit(bool closed){ if(pts_.size()<2){ QMessageBox::warning(nullptr,"Polyline","DRAW_INSUFFICIENT_INPUT"); pts_.clear(); hover_.reset(); return; }
    QByteArray doc=store_->documentJson().toUtf8(); auto d=QJsonDocument::fromJson(doc).object(); QString layer=d.value("layers").toArray().first().toObject().value("id").toString();
    QJsonArray arr; for(const auto& p: pts_) arr.append(QJsonObject{{"x",p.x},{"y",p.y}});
    QJsonObject params{{"pts",arr},{"closed",closed}};
    QByteArray lb=layer.toUtf8(), pb=QJsonDocument(params).toJson(QJsonDocument::Compact), eb=store_->epsPolicyJson().toUtf8();
    QString env=take(craftcad_history_apply_create_polyline(store_->historyHandle(),doc.constData(),lb.constData(),pb.constData(),eb.constData()));
    auto root=QJsonDocument::fromJson(env.toUtf8()).object(); if(!root.value("ok").toBool()) QMessageBox::warning(nullptr,"Polyline",QString::fromUtf8(QJsonDocument(root.value("reason").toObject()).toJson()));
    else store_->setDocumentJson(QString::fromUtf8(QJsonDocument(root.value("data").toObject().value("document").toObject()).toJson(QJsonDocument::Compact)));
    pts_.clear(); hover_.reset(); numeric_.clear();
}
void PolylineTool::onKeyPress(QKeyEvent* e){ if(e->key()==Qt::Key_H) lockH=!lockH; if(e->key()==Qt::Key_V) lockV=!lockV; if(e->key()==Qt::Key_Escape){ pts_.clear(); hover_.reset(); numeric_.clear(); }
    if(e->key()==Qt::Key_Return||e->key()==Qt::Key_Enter) commit(false);
    if(e->key()==Qt::Key_C) commit(true);
    if(e->key()==Qt::Key_Backspace && !pts_.isEmpty()) pts_.removeLast();
    numeric_.handleKey(e->key(),e->text());
}
void PolylineTool::renderOverlay(QPainter& p){ if(pts_.isEmpty()) return; p.setPen(QPen(QColor(120,255,120),1,Qt::DashLine)); for(int i=1;i<pts_.size();++i) p.drawLine(camera_->worldToScreen(pts_[i-1]),camera_->worldToScreen(pts_[i])); if(hover_) p.drawLine(camera_->worldToScreen(pts_.last()),camera_->worldToScreen(*hover_)); }
