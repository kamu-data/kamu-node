import { ComponentFixture, TestBed } from '@angular/core/testing';

import { SearchAdditionalButtonsComponent } from './search-additional-buttons.component';

describe('SearchAdditionalButtonsComponent', () => {
  let component: SearchAdditionalButtonsComponent;
  let fixture: ComponentFixture<SearchAdditionalButtonsComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [ SearchAdditionalButtonsComponent ]
    })
    .compileComponents();
  });

  beforeEach(() => {
    fixture = TestBed.createComponent(SearchAdditionalButtonsComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
