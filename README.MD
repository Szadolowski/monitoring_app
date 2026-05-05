# 🛠️ Instrukcja Zarządzania Systemem File Monitor

Poniższe zapytania SQL należy wykonywać z poziomu narzędzia **Query Tool** (np. w pgAdmin 4), po podłączeniu do bazy danych.

---

## 💡 Ważne Przypomnienie – Grupy (`user_groups`)

W bazie istnieją dwie główne grupy przypisane na stałe:

| `group_id` | Nazwa | Opis |
|---|---|---|
| `1` | **Biuro** | Wiele profili/katalogów, dostęp chroniony jednym hasłem dla grupy |
| `2` | **Brygadzista** | Dostęp do pojedynczego profilu budowy, każdy profil ma własne hasło |

---

## 👥 1. Zarządzanie Profilami (Katalogami/Osobami)

**Tabela:** `public.monitoring_profiles`

### 🟢 Dodawanie nowego profilu dla BIURA (`group_id = 1`)

Tworzy nową kategorię ścieżek dostępną w panelu "Biuro". Hasło weryfikuje dostęp do całego panelu biurowego.

```sql
INSERT INTO public.monitoring_profiles (group_id, profile_name, password_hash, description)
VALUES (
    1,
    'Księgowość i Kadry',
    crypt('HasloBiura2024!', gen_salt('bf')),
    'Katalogi z fakturami i dokumentacją pracowniczą.'
);
```

### 🟢 Dodawanie nowego profilu dla BRYGADZISTY (`group_id = 2`)

Tworzy dedykowany profil konkretnej budowy. Hasło zdefiniowane tutaj będzie wymagane przez aplikację tylko po wybraniu tej konkretnej budowy.

```sql
INSERT INTO public.monitoring_profiles (group_id, profile_name, password_hash, description)
VALUES (
    2,
    'Budowa Centrum - Jan Kowalski',
    crypt('PINElektronik123', gen_salt('bf')),
    'Projekt modernizacji centrum handlowego.'
);
```

### 🟡 Edycja danych profilu

Pozwala na zmianę nazwy lub opisu istniejącego konta. Pamiętaj, aby sprawdzić `id` profilu przed zmianą.

```sql
UPDATE public.monitoring_profiles
SET 
    profile_name = 'Budowa Centrum (ZAKOŃCZONA)',
    description = 'Projekt archiwalny, monitorowanie tylko dla celów weryfikacji.'
WHERE id = 2;
```

### 🔐 Resetowanie hasła (PIN-u)

Nadpisuje zapomniane hasło nowym, poprawnie zaszyfrowanym hashem.

```sql
UPDATE public.monitoring_profiles
SET 
    password_hash = crypt('NoweBardzoSilneHaslo99!', gen_salt('bf'))
WHERE id = 2;
```

### 🔴 Usuwanie profilu

Trwale kasuje profil z systemu. Dzięki opcji `ON DELETE CASCADE`, wszystkie przypisane do niego ścieżki również zostaną usunięte.

```sql
DELETE FROM public.monitoring_profiles
WHERE id = 2;
```

---

## 📁 2. Zarządzanie Śledzonymi Ścieżkami

**Tabela:** `public.watched_paths`

### 🟢 Dodawanie nowej ścieżki

Przypisuje folder do konkretnego profilu (`profile_id`).

```sql
INSERT INTO public.watched_paths (profile_id, path, notification_msg, is_active)
VALUES (
    2,
    'D:\Projekty\Budowa_X\Zdjęcia_Postępu',
    'Nowe zdjęcia z placu budowy! Sprawdź folder.',
    true
);
```

### ⏸️ Zawieszanie / Aktywacja śledzenia

Zamiast usuwać ścieżkę, możesz ją tymczasowo wyłączyć.

```sql
UPDATE public.watched_paths
SET is_active = false  -- zmień na true, aby włączyć ponownie
WHERE id = 5;
```

### 🔴 Usuwanie ścieżki

Trwałe usunięcie folderu z listy monitorowania.

```sql
DELETE FROM public.watched_paths
WHERE id = 5;
```
