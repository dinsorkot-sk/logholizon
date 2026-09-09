-- Automation execution actor is the system automation runtime.
ALTER TABLE _automation_execution ADD COLUMN actor TEXT NOT NULL DEFAULT 'automation';
