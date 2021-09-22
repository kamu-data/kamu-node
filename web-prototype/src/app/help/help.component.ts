import { Component, OnInit } from '@angular/core';
import AppValues from "../common/app.values";

@Component({
  selector: 'app-help',
  templateUrl: './help.component.html',
  styleUrls: ['./help.component.sass']
})
export class HelpComponent implements OnInit {
  public title = AppValues.appTitle;

  constructor() { }

  public ngOnInit(): void {
  }

}
