from __future__ import annotations
from pydantic import BaseModel, Field
from typing import Optional
from datetime import datetime
import uuid


class SignalSummary(BaseModel):
    signal_id:  str
    category:   str
    summary:    str
    was_useful: Optional[bool] = None


class PostMortem(BaseModel):
    postmortem_id:                   str   = Field(default_factory=lambda: str(uuid.uuid4()))
    generated_at:                    datetime
    market_id:                       str
    market_question:                 str
    thesis_id:                       Optional[str]   = None
    execution_id:                    Optional[str]   = None
    market_resolved_as:              str
    thesis_was_correct:              Optional[bool]  = None
    realized_pnl_usd:                Optional[float] = None
    signals_present:                 list[SignalSummary]
    what_the_thesis_said:            str
    what_happened:                   str
    what_would_have_changed_outcome: str
    source_weight_updates:           dict[str, float]  # {source_id: delta}
    vault_path:                      str

    # Redis channel
    CHANNEL: str = "oracle:post_mortem"
