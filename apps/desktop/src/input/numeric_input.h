#pragma once
#include <QString>
#include <optional>

class NumericInput {
public:
    void handleKey(int key, const QString& text);
    void clear();
    std::optional<double> value() const;
    QString buffer() const { return buffer_; }
private:
    QString buffer_;
};
