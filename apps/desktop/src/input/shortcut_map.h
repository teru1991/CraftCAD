#pragma once
#include <QKeyEvent>

inline bool isUndo(QKeyEvent* e){ return (e->modifiers() & Qt::ControlModifier) && e->key()==Qt::Key_Z && !(e->modifiers() & Qt::ShiftModifier); }
inline bool isRedo(QKeyEvent* e){ return (e->modifiers() & Qt::ControlModifier) && (e->key()==Qt::Key_Y || ((e->modifiers() & Qt::ShiftModifier) && e->key()==Qt::Key_Z)); }
