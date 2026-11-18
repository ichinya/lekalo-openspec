# lekalo-openspec

Поставщик требований [Lekalo](https://github.com/ichinya/lekalo) для OpenSpec: читает принятые требования и активные изменения, связывает их идентификаторы и редакции с символами/сценариями модели, показывает отсутствующие связи и изменившиеся требования.

- Выделяется из встроенного openspec provider ядра за общий интерфейс поставщика требований — [#1](https://github.com/ichinya/lekalo-openspec/issues/1).
- Языконезависимый: один пакет обслуживает PHP-, TypeScript- и Python-проекты; пакеты вида lekalo-php-openspec не нужны.
- Общий план отделения — [ichinya/lekalo#241](https://github.com/ichinya/lekalo/issues/241).

## Реализация

`src/` содержит перенесённый из ядра парсер OpenSpec, построение эффективного каталога, правила конфликтов и идентификаторов. Библиотека не зависит от `lekalo-core`. Хост передаёт `Source` — ограниченный read-only доступ с запретом symlink/traversal; произвольный доступ к диску библиотека не получает. Публичные схемы требований и отчётов пока остаются контрактами ядра.

Проверка: `cargo test --locked`. Интеграционные проверки filesystem confinement и прежних wire-векторов остаются в основном репозитории. Выбор provider производится через `services.requirements` в пользовательском конфиге.

`ci/package-workflow.yml` — неактивный шаблон CI. Для автоматического запуска его нужно установить в `.github/workflows/` учётными данными с правом `workflow`; текущая публикация проверена локально.

## Процессный сервис

```sh
cargo install --locked --path . --root /your/tools
```

Добавьте `/your/tools/bin` в PATH, укажите `services.requirements: {command: [lekalo-openspec]}` в `lekalo.yaml` и выполните `lekalo runtime lock`. Процесс реализует `describe` и `requirements.snapshot` в оболочке `lekalo/runtime/v1`. Ядро не компилирует этот пакет и не содержит его копии. [Схемы и ограничения](https://github.com/ichinya/lekalo/blob/feat/external-services-241/docs/external-services.md). Сборка использует `debug=0`, `incremental=false`.

После установки бинарника в PATH можно указать `gh/ichinya/lekalo-openspec` в services. При lock ядро сверяет идентичность и версию исполняемого сервиса с Git-манифестом.
