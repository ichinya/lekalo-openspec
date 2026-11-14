# lekalo-openspec

Поставщик требований [Lekalo](https://github.com/ichinya/lekalo) для OpenSpec: читает принятые требования и активные изменения, связывает их идентификаторы и редакции с символами/сценариями модели, показывает отсутствующие связи и изменившиеся требования.

- Выделяется из встроенного openspec provider ядра за общий интерфейс поставщика требований — [#1](https://github.com/ichinya/lekalo-openspec/issues/1).
- Языконезависимый: один пакет обслуживает PHP-, TypeScript- и Python-проекты; пакеты вида lekalo-php-openspec не нужны.
- Общий план отделения — [ichinya/lekalo#241](https://github.com/ichinya/lekalo/issues/241).
