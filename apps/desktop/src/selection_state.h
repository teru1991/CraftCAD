#pragma once
#include <QSet>
#include <QString>

class SelectionState {
public:
    void clear();
    void setSingle(const QString& id);
    bool isSelected(const QString& id) const;
    const QSet<QString>& ids() const;
private:
    QSet<QString> selected_;
};
