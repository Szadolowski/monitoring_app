# 🛠️ Instrukcja Zarządzania Systemem File Monitor

Poniższe zapytania SQL należy wykonywać z poziomu narzędzia **Query Tool** (np. w pgAdmin 4), po podłączeniu do bazy `monitoring_db`.

---

## 👥 1. Zarządzanie Profilami (Użytkownikami)

**Tabela:** `public.monitoring_profiles`

### 🟢 Dodawanie nowego profilu

Tworzy nowe konto w systemie.  
**Uwaga:** zawsze używaj funkcji `crypt()` do bezpiecznego hashowania kodu PIN.

```sql
INSERT INTO public.monitoring_profiles (profile_name, pin_code, description)
VALUES (
    'Księgowość',
    crypt('HasloKsiegowej123!', gen_salt('bf')),
    'Profil domyślny dla działu finansowego i księgowości.'
);
```

---

### 🟡 Edycja danych profilu

Pozwala na zmianę nazwy lub opisu istniejącego konta.  
Upewnij się, że modyfikujesz właściwe `id`.

```sql
UPDATE public.monitoring_profiles
SET
    profile_name = 'Główna Księgowość',
    description = 'Zaktualizowany opis profilu z pełnym dostępem.'
WHERE id = 1;
```

---

### 🔐 Resetowanie hasła (PIN-u)

Nadpisuje zapomniane hasło nowym, poprawnie zaszyfrowanym hashem.

```sql
UPDATE public.monitoring_profiles
SET
    pin_code = crypt('NoweZresetowaneHaslo!@#', gen_salt('bf'))
WHERE id = 1;
```

---

### 🔴 Usuwanie profilu

Trwale kasuje konto z systemu.

⚠️ **Uwaga:** upewnij się, że usunięcie profilu nie narusza powiązań kluczy obcych (foreign keys) ze śledzonymi ścieżkami, chyba że masz włączoną opcję `ON DELETE CASCADE`.

```sql
DELETE FROM public.monitoring_profiles
WHERE id = 5;
```

---

## 📁 2. Zarządzanie Śledzonymi Ścieżkami

**Tabela:** `public.watched_paths`

### 🟢 Dodawanie nowej ścieżki

Przypisuje folder do konkretnego profilu (`profile_id`) i ustala niestandardowy komunikat powiadomienia.

```sql
INSERT INTO public.watched_paths (profile_id, path, notification_msg)
VALUES (
    1,
    'C:\Wspolne\Faktury_2026',
    'Pobrano nową fakturę! Kliknij przycisk Otwórz, aby sprawdzić.'
);
```

---

### 🟡 Edycja ścieżki / komunikatu

Pozwala na zmianę lokalizacji folderu lub aktualizację tekstu powiadomienia.

```sql
UPDATE public.watched_paths
SET
    path = 'D:\Nowy_Katalog\Faktury',
    notification_msg = 'Uwaga! Nowy dokument w archiwum.'
WHERE id = 2;
```

---

### 🔴 Przerywanie śledzenia folderu

Usuwa ścieżkę z bazy, co spowoduje, że Agent przestanie monitorować katalog przy kolejnej synchronizacji.

```sql
DELETE FROM public.watched_paths
WHERE id = 2;
```
